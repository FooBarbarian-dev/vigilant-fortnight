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

    fn all_complete(&self) -> bool {
        self.tasks.iter().all(|t| self.completed.contains(t))
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

    /// Find the best worker for a task (simple heuristic based on system prompt)
    fn assign_worker<'a>(task: &str, workers: &'a [&'a Agent]) -> Option<&'a Agent> {
        // Simple heuristic: match keywords in task to system prompts
        let task_lower = task.to_lowercase();

        for worker in workers {
            let prompt_lower = worker.system_prompt().to_lowercase();

            // Check if worker's specialty matches task keywords
            if task_lower.contains("summar") && prompt_lower.contains("summar") {
                return Some(worker);
            }
            if task_lower.contains("analyz") && prompt_lower.contains("analyz") {
                return Some(worker);
            }
            if task_lower.contains("write") && prompt_lower.contains("write") {
                return Some(worker);
            }
            if task_lower.contains("review") && prompt_lower.contains("review") {
                return Some(worker);
            }
        }

        // Default to first worker
        workers.first().copied()
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

        // Step 2: Execute tasks iteratively
        for iteration in 0..self.max_iterations {
            metadata.add_trace(format!("--- Iteration {} ---", iteration + 1));

            let pending = ledger.pending_tasks();
            if pending.is_empty() {
                metadata.add_trace("All tasks complete".to_string());
                break;
            }

            metadata.add_trace(format!("Pending tasks: {}", pending.len()));

            // Work on first pending task
            let task = pending[0].clone();
            metadata.add_trace(format!("Working on: {}", task));

            // Assign to appropriate worker
            if let Some(worker) = Self::assign_worker(&task, &workers) {
                metadata.add_trace(format!("Assigned to worker: {}", worker.id()));

                tracing::debug!(
                    iteration = iteration + 1,
                    task = %task,
                    worker_id = %worker.id(),
                    "Magentic: worker executing task"
                );

                match worker.prompt(&task).await {
                    Ok(result) => {
                        metadata.add_trace(format!(
                            "Worker '{}' completed task ({} chars)",
                            worker.id(),
                            result.len()
                        ));
                        task_results.push((task.clone(), result));
                        ledger.complete_task(&task);
                    }
                    Err(e) => {
                        metadata.add_trace(format!("Worker '{}' failed: {}", worker.id(), e));
                        // Continue with other tasks
                    }
                }
            }

            // Check if manager thinks we're done
            if ledger.all_complete() {
                break;
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
        assert!(!ledger.all_complete());

        ledger.complete_task("task1");
        assert_eq!(ledger.pending_tasks().len(), 1);
        assert!(!ledger.all_complete());

        ledger.complete_task("task2");
        assert_eq!(ledger.pending_tasks().len(), 0);
        assert!(ledger.all_complete());
    }
}
