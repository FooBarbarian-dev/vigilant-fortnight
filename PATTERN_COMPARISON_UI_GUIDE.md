# Pattern Comparison UI - User Guide

## Overview

The new Pattern Comparison UI executes **all 5 orchestration patterns in parallel** and displays results in separate tabs with real-time DAG visualization.

## ✅ What's Fixed

1. **✅ Tab-Based Interface** - One tab per pattern (Sequential, Concurrent, Group Chat, Handoff, Magentic)
2. **✅ Expected DAG Visualization** - See the execution flow BEFORE running
3. **✅ Execute All Button** - Single button executes all patterns at once
4. **✅ Removed Individual Execute** - No more confusion with multiple buttons
5. **✅ Fixed WebSocket Errors** - Connection stays alive throughout all pattern executions
6. **✅ Provider Tracking** - Each agent shows which LLM provider is being used (OpenAI/Anthropic/Cohere)
7. **✅ Pattern-Tagged Logs** - All backend logs show `[pattern_id]` prefix for filtering

## How to Use

### 1. Start the Server

```bash
cd rig-patterns-ui
cargo run --release
```

Navigate to `http://localhost:3009`

### 2. Configure Agents Per Pattern

The UI has **5 tabs**, one for each pattern. Each tab has its own agent configuration:

**Sequential Tab:**
- Agent 1 → Agent 2 → Agent 3 (chain)
- Each agent receives the previous agent's output

**Concurrent Tab:**
- All agents receive the same input
- Results are aggregated (Combine/Vote)

**Group Chat Tab:**
- Agents discuss in rounds
- Continues until CONSENSUS_REACHED or max rounds

**Handoff Tab:**
- Agents route tasks to each other
- Uses HANDOFF:agent_id markers

**Magentic Tab:**
- First agent is the Manager
- Others are Workers
- Manager breaks down tasks and synthesizes results

### 3. Edit Agent Configurations

For each pattern tab:

1. Click the tab (e.g., "SEQUENTIAL")
2. You'll see the expected DAG visualization
3. Scroll to "Agent Configuration" section
4. Edit existing agents or click "+ ADD AGENT"

Each agent has:
- **ID**: Unique identifier (e.g., `agent1`, `reviewer`, `triage`)
- **Provider**: OpenAI, Anthropic, or Cohere
- **Model**: Model name (e.g., `gpt-4`, `claude-3-opus-20240229`)
- **System Prompt**: Role/instructions for the agent

**DAG Updates Live:** When you add/remove/edit agents, the DAG automatically updates to show the new flow!

### 4. Enter Root Prompt

At the top, there's a **ROOT NEURAL PROMPT** text area. This is the input that ALL patterns will process.

Example:
```
Explain the benefits of Rust for systems programming
```

### 5. Click "EXECUTE ALL PATTERNS"

This single button:
1. Sends a `CompareRequest` to the backend via WebSocket
2. Executes all 5 patterns **in parallel**
3. Streams events in real-time to the correct tabs

### 6. Watch Execution in Tabs

Switch between tabs to see each pattern's execution:

- **Status Badge**: Shows READY → RUNNING → COMPLETE
- **Execution Log**: Real-time log entries showing:
  - Agent receives input
  - Agent thinking
  - Agent responds
  - Handoffs (for Handoff pattern)
  - Consensus messages (for Group Chat)
  - Task assignments (for Magentic)
- **DAG**: The DAG nodes highlight as agents execute (future enhancement)

### 7. Review Results

After execution completes:
- Each tab shows the final output in its execution log
- Status badges turn green (COMPLETE) or red (ERROR)
- WebSocket automatically closes

## Expected DAG Examples

### Sequential Pattern
```
Input → Agent1[openai] → Agent2[openai] → Output
```

### Concurrent Pattern
```
         ┌→ Agent1[openai] ┐
Input →──┤                  ├→ Aggregator → Output
         └→ Agent2[openai] ┘
```

### Group Chat Pattern
```
         ┌→ Agent1[openai] ┐
Round1 →─┤                  ├→ Consensus? → Output (or Round2...)
         └→ Agent2[openai] ┘
```

### Handoff Pattern
```
Input → Agent1[openai] →[HANDOFF?]→ Agent2[openai] → Output
```

### Magentic Pattern
```
Input → Manager[openai] → Task1 → Worker1[openai] ┐
                        ↓ Task2 → Worker2[openai] ├→ Synthesize → Manager → Output
```

## Backend Logging

All backend logs now include pattern identifiers:

```
[sequential] 🚀 REAL EXECUTION - Making actual LLM API calls
[sequential][Sequential Step 1/2] Agent: agent1 (openai)
[concurrent] Starting CONCURRENT pattern with 2 agents
[group_chat] Consensus reached at round 1
[handoff] Handoff pattern completed after 0 hops
[magentic] Manager synthesizing all results...
```

**To filter logs:**
```bash
# Watch sequential pattern only
cargo run --release 2>&1 | grep "\[sequential\]"

# Watch all patterns' completions
cargo run --release 2>&1 | grep "completed"
```

## WebSocket Protocol

The UI sends this request:

```json
{
  "input": "Your root prompt here",
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
    "concurrent": { ... },
    "group_chat": { ... },
    "handoff": { ... },
    "magentic": { ... }
  }
}
```

The backend responds with events like:

```json
{
  "type": "agent_responds",
  "pattern_id": "sequential",
  "agent_id": "agent1",
  "provider": "openai",
  "response": "Rust provides memory safety...",
  "timestamp": "2025-11-10T01:57:56.610774Z"
}
```

## Environment Variables

Make sure you have API keys set:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export COHERE_API_KEY="..."
```

The UI will use the appropriate key based on the `provider` field in each agent config.

## Troubleshooting

### WebSocket Errors

**Error:** `Broken pipe (os error 32)`

**Cause:** WebSocket closing prematurely

**Fix:** ✅ This is now fixed! The new UI keeps the WebSocket open until all patterns complete.

### Pattern Not Executing

**Error:** Pattern tab shows READY but never RUNNING

**Check:**
1. Is the WebSocket connected? (Green dot in header)
2. Are there agents configured for that pattern?
3. Check browser console for JavaScript errors

### DAG Not Updating

**Issue:** Adding agents doesn't update the DAG

**Fix:**
1. Make sure Mermaid.js loaded (check browser console)
2. Try refreshing the page
3. Check that you're editing the correct tab

### No LLM Responses

**Issue:** Agent shows "THINKING..." but never responds

**Check:**
1. API keys are set correctly
2. Check server logs for LLM API errors
3. Verify the model name is correct (e.g., `gpt-4` not `gpt4`)

## Architecture

```
┌─────────────────────────────────────────────────┐
│  Browser (index.html + app.js)                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐        │
│  │Sequential│ │Concurrent│ │GroupChat │ ...    │
│  │   Tab    │ │   Tab    │ │   Tab    │        │
│  └─────┬────┘ └─────┬────┘ └─────┬────┘        │
│        │            │            │              │
│        └────────────┴────────────┘              │
│                     │                           │
│              [WebSocket Client]                 │
└─────────────────────┬───────────────────────────┘
                      │
                 WebSocket
                      │
┌─────────────────────┴───────────────────────────┐
│  Server (Rust - Axum)                           │
│  ┌─────────────────────────────────────┐        │
│  │  execute_all_patterns()             │        │
│  │  ┌──────────┐ ┌──────────┐         │        │
│  │  │Sequential│ │Concurrent│ ...     │        │
│  │  │ (spawn)  │ │ (spawn)  │         │        │
│  │  └────┬─────┘ └────┬─────┘         │        │
│  │       │            │               │        │
│  │       └────────────┴──────┐        │        │
│  │                            │        │        │
│  │                     [Event Channel] │        │
│  └────────────────────────────┬────────┘        │
│                               │                 │
│                        [WebSocket Send]         │
└───────────────────────────────┬─────────────────┘
                                │
                          (Back to Browser)
```

## Next Steps

### Unit Tests (For 80% Coverage)

See `FRONTEND_ROADMAP.md` for test implementation details.

### Enhanced DAG Visualization

Future enhancement: Highlight active nodes during execution

### Save/Load Configurations

Future enhancement: Save pattern configurations and reload them later

## Files Modified

- `rig-patterns-ui/static/index.html` - Complete tab-based redesign
- `rig-patterns-ui/static/app.js` - New event routing and DAG rendering
- `rig-patterns-ui/static/styles.css` - Tab and panel styles
- `rig-patterns-ui/src/routes/websocket.rs` - Parallel execution with pattern IDs
- `rig-patterns-ui/src/state.rs` - CompareRequest and event structure

## Comparison: Old vs New

### Old UI
- ❌ Single pattern dropdown
- ❌ "Execute" and "Compare All" buttons (confusing)
- ❌ Compare All used REST API (no real-time updates)
- ❌ No DAG visualization
- ❌ Shared log for all patterns
- ❌ WebSocket closed after single pattern

### New UI
- ✅ 5 separate tabs (one per pattern)
- ✅ Single "Execute All Patterns" button
- ✅ WebSocket streaming with CompareRequest
- ✅ Expected DAG shown before execution
- ✅ Per-pattern execution logs
- ✅ WebSocket stays alive for all patterns
- ✅ Provider tracking in all events
- ✅ Pattern IDs in all logs

## Summary

You now have a **complete pattern comparison system** that:

1. Shows expected execution flow (DAG) before running
2. Executes all patterns in parallel with a single button
3. Tracks which LLM provider each agent uses
4. Routes events to the correct tab based on pattern_id
5. Keeps WebSocket alive throughout execution
6. Provides comprehensive logging with pattern identifiers

Just set your API keys, configure agents, and click "EXECUTE ALL PATTERNS"! 🚀
