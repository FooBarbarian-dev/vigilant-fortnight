# Frontend Roadmap for Pattern Comparison UI

## ✅ Completed (Backend)

### Core Infrastructure
- [x] Pattern IDs added to all execution events
- [x] Provider tracking (OpenAI/Anthropic/Cohere) in all events
- [x] Parallel pattern execution (`execute_all_patterns()`)
- [x] Structured logging with `[pattern_id]` prefixes
- [x] WebSocket API for real-time comparison
- [x] Real LLM API integration (no mocking)

### Pattern Implementations
- [x] Sequential: Chain execution with real responses
- [x] Concurrent: Parallel async execution
- [x] GroupChat: Multi-round conversations
- [x] Handoff: Agent-to-agent routing
- [x] Magentic: Manager-worker task breakdown

## 🚧 TODO (Frontend)

### 1. Tab-Based UI (High Priority)

**File**: `rig-patterns-ui/static/index.html`

Create tabs for each pattern:
```html
<div class="pattern-tabs">
  <button class="tab-btn active" data-pattern="sequential">Sequential</button>
  <button class="tab-btn" data-pattern="concurrent">Concurrent</button>
  <button class="tab-btn" data-pattern="group_chat">Group Chat</button>
  <button class="tab-btn" data-pattern="handoff">Handoff</button>
  <button class="tab-btn" data-pattern="magentic">Magentic</button>
</div>

<div class="tab-content">
  <div class="pattern-panel active" id="sequential-panel">
    <!-- DAG visualization + logs -->
  </div>
  <!-- ... other panels ... -->
</div>
```

**CSS**: Add tab styling with active states, panel switching animations

**JS** (`static/app.js`):
- Tab click handlers to switch between patterns
- Route WebSocket events to correct tab based on `event.pattern_id`
- Update active tab indicator

### 2. DAG Visualization (High Priority)

**Library Options**:
- **Mermaid.js** (recommended): Simple, declarative, good for flowcharts
- **D3.js**: More powerful but complex
- **Cytoscape.js**: Graph visualization

**Implementation**:
```javascript
// For each pattern panel
function updateDAG(patternId, event) {
  const dagContainer = document.querySelector(`#${patternId}-dag`);

  // Add node for agent
  if (event.type === 'agent_receives_input') {
    addNode(dagContainer, {
      id: event.agent_id,
      label: `${event.agent_id}\n(${event.provider})`,
      status: 'active'
    });
  }

  // Add edge for handoff
  if (event.type === 'agent_handoff') {
    addEdge(dagContainer, {
      from: event.from_agent,
      to: event.to_agent,
      label: 'handoff'
    });
  }

  // Update node status
  if (event.type === 'agent_responds') {
    updateNodeStatus(dagContainer, event.agent_id, 'completed');
  }
}
```

**Mermaid Example**:
```html
<div class="mermaid" id="sequential-dag">
graph LR
  A[Agent1<br/>OpenAI] -->|output| B[Agent2<br/>OpenAI]
  B -->|output| C[Agent3<br/>Anthropic]

  style A fill:#0f0
  style B fill:#ff0
  style C fill:#00f
</div>
```

### 3. Editable System Prompts (High Priority)

**UI Structure**:
```html
<div class="prompt-editor">
  <h3>Root Prompt</h3>
  <textarea id="root-prompt" placeholder="Enter your task..."></textarea>

  <h3>Agent System Prompts</h3>
  <div class="pattern-specific-prompts">
    <!-- Tabs for each pattern -->
    <div class="pattern-prompts" data-pattern="sequential">
      <div class="agent-prompt">
        <label>Agent 1 (Sequential)</label>
        <input type="text" class="agent-id" value="agent1" />
        <select class="provider">
          <option value="openai">OpenAI</option>
          <option value="anthropic">Anthropic</option>
          <option value="cohere">Cohere</option>
        </select>
        <select class="model">
          <option value="gpt-4">GPT-4</option>
          <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
        </select>
        <textarea class="system-prompt" placeholder="System prompt..."></textarea>
      </div>
      <!-- More agents... -->
    </div>
    <!-- Other patterns... -->
  </div>

  <button id="execute-all">EXECUTE ALL PATTERNS</button>
</div>
```

**JavaScript**:
```javascript
function buildCompareRequest() {
  const rootPrompt = document.getElementById('root-prompt').value;

  const patternConfigs = {};
  ['sequential', 'concurrent', 'group_chat', 'handoff', 'magentic'].forEach(pattern => {
    const agents = [];
    document.querySelectorAll(`[data-pattern="${pattern}"] .agent-prompt`).forEach(promptEl => {
      agents.push({
        id: promptEl.querySelector('.agent-id').value,
        provider: promptEl.querySelector('.provider').value,
        model: promptEl.querySelector('.model').value,
        system_prompt: promptEl.querySelector('.system-prompt').value
      });
    });

    patternConfigs[pattern] = {
      pattern: getPatternConfig(pattern),
      agents: agents
    };
  });

  return {
    input: rootPrompt,
    pattern_configs: patternConfigs
  };
}

document.getElementById('execute-all').addEventListener('click', () => {
  const request = buildCompareRequest();
  ws.send(JSON.stringify(request));
});
```

### 4. Enhanced Event Handling (Medium Priority)

**Update WebSocket handler**:
```javascript
function handleStreamingEvent(event) {
  const patternId = event.pattern_id;
  const panel = document.querySelector(`#${patternId}-panel`);

  // Update DAG
  updateDAG(patternId, event);

  // Update execution log
  addLogEntry(panel.querySelector('.execution-log'), event);

  // Update status indicator
  if (event.type === 'pattern_complete') {
    updateTabStatus(patternId, 'completed');
  } else if (event.type === 'pattern_error') {
    updateTabStatus(patternId, 'error');
  }
}
```

### 5. Status Indicators (Medium Priority)

Show execution status for each pattern:
```html
<button class="tab-btn" data-pattern="sequential">
  Sequential
  <span class="status-indicator running"></span>
</button>
```

CSS states:
- `pending`: Gray dot
- `running`: Yellow pulsing dot
- `completed`: Green checkmark
- `error`: Red X

### 6. Execution Logs Per Pattern (Medium Priority)

Each panel should have its own log:
```html
<div class="pattern-panel" id="sequential-panel">
  <div class="dag-container"><!-- DAG here --></div>
  <div class="execution-log">
    <h4>Execution Log</h4>
    <div class="log-entries">
      <!-- Auto-scrolling log entries -->
    </div>
  </div>
</div>
```

Filter logs by pattern_id from WebSocket events.

### 7. Unit Tests (Required for 80% Coverage)

**Backend Tests** (`rig-patterns-ui/tests/`):

```rust
// tests/websocket_tests.rs
#[tokio::test]
async fn test_parallel_pattern_execution() {
    // Test that all 5 patterns execute in parallel
}

#[tokio::test]
async fn test_pattern_id_in_events() {
    // Verify all events include pattern_id
}

#[tokio::test]
async fn test_provider_tracking() {
    // Verify provider field is set correctly
}
```

**Frontend Tests** (optional, using Jest):
```javascript
// static/tests/app.test.js
describe('Pattern Tabs', () => {
  test('switches active tab on click', () => {
    // ...
  });

  test('routes events to correct panel', () => {
    // ...
  });
});
```

**Coverage Tools**:
- Backend: `cargo tarpaulin --out Html`
- Frontend: `jest --coverage`

## Implementation Priority

### Phase 1: Core UI (Week 1)
1. Tab-based layout
2. Basic panel switching
3. Event routing by pattern_id
4. Execute button with CompareRequest

### Phase 2: Visualization (Week 2)
1. Integrate Mermaid.js or D3.js
2. Implement DAG rendering per pattern
3. Real-time DAG updates from events
4. Provider labels on nodes

### Phase 3: Editing (Week 3)
1. System prompt editor UI
2. Per-pattern agent configuration
3. Save/load configurations
4. Validation

### Phase 4: Polish (Week 4)
1. Status indicators
2. Enhanced logging
3. Error handling
4. Unit tests for 80% coverage

## Testing the Current Backend

You can test the parallel execution now:

```javascript
// In browser console or via WebSocket client
const ws = new WebSocket('ws://localhost:3009/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({
    input: "Explain quantum computing",
    pattern_configs: null  // Uses default agents
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(`[${data.pattern_id}] ${data.type}`, data);
};
```

All 5 patterns will execute in parallel and stream events with pattern IDs!

## Files to Modify

1. `rig-patterns-ui/static/index.html` - Tab structure
2. `rig-patterns-ui/static/styles.css` - Tab styling, DAG styles
3. `rig-patterns-ui/static/app.js` - Tab logic, DAG rendering, event routing
4. `rig-patterns-ui/tests/*.rs` - Unit tests
5. `rig-patterns-ui/Cargo.toml` - Add test dependencies if needed

## Resources

- [Mermaid.js Docs](https://mermaid.js.org/)
- [D3.js Force Directed Graph](https://d3-graph-gallery.com/network.html)
- [Cargo Tarpaulin](https://github.com/xd009642/tarpaulin)
- [WebSocket API Spec](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
