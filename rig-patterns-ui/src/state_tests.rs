#[cfg(test)]
mod tests {
    use crate::state::{CompareRequest, PatternConfig, AgentConfig};

    #[test]
    fn test_parse_compare_request() {
        let json = r#"{
            "input": "Tell me about Rust",
            "pattern_configs": {
                "sequential": {
                    "pattern": { "type": "sequential" },
                    "agents": [
                        {
                            "id": "agent1",
                            "provider": "openai",
                            "model": "gpt-4",
                            "system_prompt": "You are helpful."
                        }
                    ]
                },
                "concurrent": {
                    "pattern": {
                        "type": "concurrent",
                        "aggregation": "combine"
                    },
                    "agents": [
                        {
                            "id": "agent1",
                            "provider": "openai",
                            "model": "gpt-4",
                            "system_prompt": "You are helpful."
                        }
                    ]
                }
            }
        }"#;

        let result: Result<CompareRequest, _> = serde_json::from_str(json);
        match &result {
            Ok(req) => {
                println!("✅ Successfully parsed CompareRequest");
                println!("Input: {}", req.input);
                if let Some(configs) = &req.pattern_configs {
                    println!("Pattern configs: {}", configs.len());
                }
            }
            Err(e) => {
                println!("❌ Failed to parse: {}", e);
            }
        }

        assert!(result.is_ok(), "Failed to parse CompareRequest: {:?}", result.err());
    }

    #[test]
    fn test_parse_pattern_configs() {
        // Test each pattern type individually

        // Sequential
        let seq_json = r#"{"type": "sequential"}"#;
        let seq: Result<PatternConfig, _> = serde_json::from_str(seq_json);
        assert!(seq.is_ok(), "Sequential parse failed: {:?}", seq.err());

        // Concurrent
        let conc_json = r#"{"type": "concurrent", "aggregation": "combine"}"#;
        let conc: Result<PatternConfig, _> = serde_json::from_str(conc_json);
        assert!(conc.is_ok(), "Concurrent parse failed: {:?}", conc.err());

        // GroupChat
        let gc_json = r#"{"type": "group_chat", "max_rounds": 3}"#;
        let gc: Result<PatternConfig, _> = serde_json::from_str(gc_json);
        assert!(gc.is_ok(), "GroupChat parse failed: {:?}", gc.err());

        // Handoff
        let ho_json = r#"{"type": "handoff", "max_hops": 5}"#;
        let ho: Result<PatternConfig, _> = serde_json::from_str(ho_json);
        assert!(ho.is_ok(), "Handoff parse failed: {:?}", ho.err());

        // Magentic
        let mag_json = r#"{"type": "magentic", "max_iterations": 5}"#;
        let mag: Result<PatternConfig, _> = serde_json::from_str(mag_json);
        assert!(mag.is_ok(), "Magentic parse failed: {:?}", mag.err());
    }

    #[test]
    fn test_parse_agent_config() {
        let json = r#"{
            "id": "test-agent",
            "provider": "openai",
            "model": "gpt-4",
            "system_prompt": "You are a test agent."
        }"#;

        let result: Result<AgentConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "AgentConfig parse failed: {:?}", result.err());
    }
}
