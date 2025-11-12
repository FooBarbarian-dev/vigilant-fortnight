//! Magentic pattern: manager agent coordinates workers on a task list

use super::{PatternExecutor, PatternMetadata};
use crate::{Agent, OrchestratorResult};
use async_trait::async_trait;
use std::collections::HashSet;

/// Simple task ledger for tracking work
#[derive(Debug, Clone)]
struct SimpleLedger {
    tasks: Vec<String>,
    completed: HashSet<String>,
}

impl SimpleLedger {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            completed: HashSet::new(),
        }
    }

    fn add_task(&mut self, task: String) {
        if !self.tasks.contains(&task) {
            self.tasks.push(task);
        }
    }

    fn complete_task(&mut self, task: &str) {
        self.completed.insert(task.to_string());
    }

    fn pending_tasks(&self) -> Vec<&String> {
        self.tasks
            .iter()
            .filter(|t| !self.completed.contains(*t))
            .collect()
    }
}

/// Magentic pattern executor
///
/// A manager agent (first in the list) coordinates worker agents.
/// The manager builds a task list and assigns work to workers.
pub(crate) struct MagenticExecutor {
    max_iterations: usize,
}

impl MagenticExecutor {
    pub fn new(max_iterations: usize) -> Self {
        Self { max_iterations }
    }

    /// Parse tasks from manager's response
    fn parse_tasks(response: &str) -> Vec<String> {
        let mut tasks = Vec::new();

        for line in response.lines() {
            let trimmed = line.trim();

            // Look for task markers: "- ", "* ", numbers "1. ", "2. ", etc.
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                tasks.push(trimmed[2..].trim().to_string());
            } else if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_numeric()) {
                if let Some(task) = rest.strip_prefix(". ").or_else(|| rest.strip_prefix(") ")) {
                    tasks.push(task.trim().to_string());
                }
            } else if trimmed.starts_with("TASK:") {
                tasks.push(trimmed[5..].trim().to_string());
            }
        }

        tasks
    }

    /// Check if manager says work is complete
    #[cfg(test)]
    fn check_completion(response: &str) -> bool {
        let upper = response.to_uppercase();
        upper.contains("COMPLETE")
            || upper.contains("DONE")
            || upper.contains("FINISHED")
            || upper.contains("ALL TASKS COMPLETE")
    }
}

#[async_trait]
impl PatternExecutor for MagenticExecutor {
    async fn execute(&self, agents: &[Agent], input: &str) -> anyhow::Result<OrchestratorResult> {
        if agents.len() < 2 {
            anyhow::bail!("Magentic pattern requires at least 2 agents (1 manager + workers)");
        }

        if self.max_iterations == 0 {
            anyhow::bail!("Magentic pattern requires max_iterations > 0");
        }

        tracing::info!(
            "Executing magentic pattern with {} agents, max {} iterations",
            agents.len(),
            self.max_iterations
        );

        let mut metadata = PatternMetadata::new();
        metadata.add_detail("pattern", "magentic");
        metadata.add_detail("agent_count", agents.len().to_string());
        metadata.add_detail("max_iterations", self.max_iterations.to_string());

        // First agent is manager, rest are workers
        let manager = &agents[0];
        let workers: Vec<&Agent> = agents[1..].iter().collect();

        metadata.add_trace(format!(
            "Manager: '{}', Workers: {}",
            manager.id(),
            workers.iter().map(|w| w.id()).collect::<Vec<_>>().join(", ")
        ));

        let mut ledger = SimpleLedger::new();
        let mut task_results: Vec<(String, String)> = Vec::new();

        // Step 1: Manager creates task list
        metadata.add_trace("Asking manager to create task list".to_string());

        let manager_prompt = format!(
            "{}\n\nPlease break this down into specific tasks. \
            Format each task on a new line starting with '- ' or numbered.\n\
            Available workers: {}",
            input,
            workers
                .iter()
                .map(|w| format!("{} ({})", w.id(), w.system_prompt()))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let task_list_response = manager.prompt(&manager_prompt).await?;
        let tasks = Self::parse_tasks(&task_list_response);

        if tasks.is_empty() {
            anyhow::bail!("Manager failed to create any tasks");
        }

        metadata.add_trace(format!("Manager created {} tasks", tasks.len()));
        for task in &tasks {
            ledger.add_task(task.clone());
            metadata.add_trace(format!("  - {}", task));
        }

        metadata.add_detail("tasks_created", tasks.len().to_string());

        // Step 2: Execute tasks in parallel using tokio::spawn (thread pool)
        // Process all tasks at once if we have enough workers, otherwise batch them
        let pending = ledger.pending_tasks();
        let tasks_to_process: Vec<String> = pending.iter()
            .take(self.max_iterations)
            .map(|s| s.to_string())
            .collect();

        metadata.add_trace(format!("Processing {} tasks in parallel on thread pool", tasks_to_process.len()));

        if !tasks_to_process.is_empty() && !workers.is_empty() {
            // Spawn tasks for all workers on the thread pool
            let mut task_handles = Vec::new();

            for (idx, task) in tasks_to_process.iter().enumerate() {
                let task_clone = task.clone();
                let worker = workers[idx % workers.len()].clone();
                let worker_id = worker.id().to_string();

                metadata.add_trace(format!("Spawning task: {} -> worker: {}", task, worker_id));

                let handle = tokio::spawn(async move {
                    tracing::debug!(
                        task = %task_clone,
                        worker_id = %worker_id,
                        "Magentic: worker executing task on thread pool"
                    );

                    let result = worker.prompt(&task_clone).await;

                    (task_clone, worker_id, result)
                });

                task_handles.push(handle);
            }

            // Collect results from all tasks
            for handle in task_handles {
                match handle.await {
                    Ok((task, worker_id, Ok(result))) => {
                        metadata.add_trace(format!(
                            "Worker '{}' completed task '{}' ({} chars)",
                            worker_id,
                            task,
                            result.len()
                        ));
                        task_results.push((task.clone(), result));
                        ledger.complete_task(&task);
                    }
                    Ok((task, worker_id, Err(e))) => {
                        metadata.add_trace(format!("Worker '{}' failed on task '{}': {}", worker_id, task, e));
                        // Continue with other tasks
                    }
                    Err(e) => {
                        metadata.add_trace(format!("Task join error: {}", e));
                    }
                }
            }
        }

        metadata.add_detail("tasks_completed", ledger.completed.len().to_string());
        metadata.add_detail("tasks_pending", ledger.pending_tasks().len().to_string());

        // Step 3: Manager synthesizes final result
        metadata.add_trace("Asking manager to synthesize results".to_string());

        let synthesis_prompt = format!(
            "Original request: {}\n\nTask results:\n{}\n\nPlease synthesize these results into a final response.",
            input,
            task_results
                .iter()
                .map(|(task, result)| format!("Task: {}\nResult: {}\n", task, result))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let final_output = manager.prompt(&synthesis_prompt).await?;

        metadata.add_trace("Manager synthesized final result".to_string());

        let output = format!(
            "MAGENTIC ORCHESTRATION RESULT:\n\n\
            Tasks completed: {}/{}\n\n\
            --- Final Synthesis ---\n{}",
            ledger.completed.len(),
            ledger.tasks.len(),
            final_output
        );

        Ok(OrchestratorResult::new(output, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tasks() {
        let response = "Here are the tasks:\n- Task one\n- Task two\n* Task three";
        let tasks = MagenticExecutor::parse_tasks(response);
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0], "Task one");
        assert_eq!(tasks[2], "Task three");

        let numbered = "1. First task\n2. Second task";
        let tasks = MagenticExecutor::parse_tasks(numbered);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0], "First task");
    }

    #[test]
    fn test_check_completion() {
        assert!(MagenticExecutor::check_completion("All tasks COMPLETE"));
        assert!(MagenticExecutor::check_completion("We're DONE here"));
        assert!(MagenticExecutor::check_completion("Finished!"));
        assert!(!MagenticExecutor::check_completion("Still working..."));
    }

    #[test]
    fn test_ledger() {
        let mut ledger = SimpleLedger::new();
        ledger.add_task("task1".to_string());
        ledger.add_task("task2".to_string());

        assert_eq!(ledger.pending_tasks().len(), 2);

        ledger.complete_task("task1");
        assert_eq!(ledger.pending_tasks().len(), 1);

        ledger.complete_task("task2");
        assert_eq!(ledger.pending_tasks().len(), 0);
    }
}
