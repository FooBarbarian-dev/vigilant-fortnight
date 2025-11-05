// ========================================
// rig-patterns UI Application Logic
// ========================================

// State management
let state = {
    agents: [],
    pattern: 'sequential',
    patternOptions: {},
    presets: [],
    ws: null,
};

// ========================================
// Initialization
// ========================================

document.addEventListener('DOMContentLoaded', async () => {
    console.log('Initializing rig-patterns UI...');

    // Load presets
    await loadPresets();

    // Initialize with default agents
    addDefaultAgents();

    // Set up event listeners
    setupEventListeners();

    // Update pattern options
    updatePatternOptions();

    console.log('Initialization complete!');
});

// ========================================
// Preset Loading
// ========================================

async function loadPresets() {
    try {
        const response = await fetch('/api/presets');
        const presets = await response.json();
        state.presets = presets;

        const select = document.getElementById('load-preset');
        presets.forEach(preset => {
            const option = document.createElement('option');
            option.value = preset.name;
            option.textContent = preset.name;
            select.appendChild(option);
        });

        console.log(`Loaded ${presets.length} presets`);
    } catch (error) {
        console.error('Failed to load presets:', error);
    }
}

// ========================================
// Agent Management
// ========================================

function addDefaultAgents() {
    addAgent('writer', 'openai', 'gpt-4', 'You write clear, concise content.');
    addAgent('editor', 'openai', 'gpt-4', 'You edit for clarity and correctness.');
}

function addAgent(id = `agent-${state.agents.length + 1}`, provider = 'openai', model = 'gpt-4', prompt = 'You are a helpful assistant.') {
    const template = document.getElementById('agent-card-template');
    const clone = template.content.cloneNode(true);

    const card = clone.querySelector('.agent-card');
    const idInput = clone.querySelector('.agent-id');
    const providerSelect = clone.querySelector('.agent-provider');
    const modelInput = clone.querySelector('.agent-model');
    const promptTextarea = clone.querySelector('.agent-prompt');
    const removeBtn = clone.querySelector('.remove-agent');

    // Set values
    idInput.value = id;
    providerSelect.value = provider;
    modelInput.value = model;
    promptTextarea.value = prompt;

    // Remove button handler
    removeBtn.addEventListener('click', () => {
        card.remove();
        updateState();
    });

    // Update state on changes
    [idInput, providerSelect, modelInput, promptTextarea].forEach(el => {
        el.addEventListener('input', updateState);
        el.addEventListener('change', updateState);
    });

    document.getElementById('agents-container').appendChild(clone);
    updateState();
}

function getAgentsFromUI() {
    const agents = [];
    const cards = document.querySelectorAll('.agent-card');

    cards.forEach(card => {
        agents.push({
            id: card.querySelector('.agent-id').value,
            provider: card.querySelector('.agent-provider').value,
            model: card.querySelector('.agent-model').value,
            system_prompt: card.querySelector('.agent-prompt').value,
        });
    });

    return agents;
}

// ========================================
// Pattern Configuration
// ========================================

function updatePatternOptions() {
    const patternType = document.getElementById('pattern-type').value;
    const optionsContainer = document.getElementById('pattern-options');

    optionsContainer.innerHTML = '';

    switch (patternType) {
        case 'concurrent':
            optionsContainer.innerHTML = `
                <label for="aggregation">Aggregation:</label>
                <select id="aggregation" class="select-input">
                    <option value="vote">Vote</option>
                    <option value="consensus">Consensus</option>
                    <option value="combine">Combine</option>
                </select>
            `;
            break;

        case 'group_chat':
            optionsContainer.innerHTML = `
                <label for="max-rounds">Max Rounds:</label>
                <input type="number" id="max-rounds" class="select-input" value="5" min="1" max="20" style="width: 80px;">
            `;
            break;

        case 'handoff':
            optionsContainer.innerHTML = `
                <label for="max-hops">Max Hops:</label>
                <input type="number" id="max-hops" class="select-input" value="10" min="1" max="50" style="width: 80px;">
            `;
            break;

        case 'magentic':
            optionsContainer.innerHTML = `
                <label for="max-iterations">Max Iterations:</label>
                <input type="number" id="max-iterations" class="select-input" value="10" min="1" max="50" style="width: 80px;">
            `;
            break;
    }

    state.pattern = patternType;
    updateState();
}

function getPatternConfig() {
    const patternType = state.pattern;

    switch (patternType) {
        case 'sequential':
            return { type: 'sequential' };

        case 'concurrent': {
            const aggregation = document.getElementById('aggregation')?.value || 'vote';
            return {
                type: 'concurrent',
                aggregation: aggregation,
            };
        }

        case 'group_chat': {
            const maxRounds = parseInt(document.getElementById('max-rounds')?.value || '5', 10);
            return {
                type: 'group_chat',
                max_rounds: maxRounds,
            };
        }

        case 'handoff': {
            const maxHops = parseInt(document.getElementById('max-hops')?.value || '10', 10);
            return {
                type: 'handoff',
                max_hops: maxHops,
            };
        }

        case 'magentic': {
            const maxIterations = parseInt(document.getElementById('max-iterations')?.value || '10', 10);
            return {
                type: 'magentic',
                max_iterations: maxIterations,
            };
        }

        default:
            return { type: 'sequential' };
    }
}

// ========================================
// Event Listeners Setup
// ========================================

function setupEventListeners() {
    // Add agent button
    document.getElementById('add-agent').addEventListener('click', () => {
        addAgent();
    });

    // Pattern type change
    document.getElementById('pattern-type').addEventListener('change', updatePatternOptions);

    // Execute button
    document.getElementById('execute').addEventListener('click', executePattern);

    // Execute with streaming button
    document.getElementById('execute-stream').addEventListener('click', executeWithStreaming);

    // Compare all button
    document.getElementById('compare-all').addEventListener('click', compareAllPatterns);

    // Load preset
    document.getElementById('load-preset').addEventListener('change', (e) => {
        const presetName = e.target.value;
        if (presetName) {
            loadPreset(presetName);
        }
    });
}

// ========================================
// State Updates
// ========================================

function updateState() {
    state.agents = getAgentsFromUI();
    state.patternOptions = getPatternConfig();
}

// ========================================
// Pattern Execution
// ========================================

async function executePattern() {
    console.log('Executing pattern...');

    updateState();

    const input = document.getElementById('user-input').value.trim();

    if (!input) {
        alert('Please enter an input prompt');
        return;
    }

    if (state.agents.length === 0) {
        alert('Please add at least one agent');
        return;
    }

    // Show loading
    showVisualization('loading');

    try {
        const response = await fetch('/api/execute', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                agents: state.agents,
                pattern: state.patternOptions,
                input: input,
            }),
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const result = await response.json();
        console.log('Execution result:', result);

        // Show results
        displayResults(result);
    } catch (error) {
        console.error('Execution failed:', error);
        alert(`Execution failed: ${error.message}`);
        hideVisualization();
    }
}

async function executeWithStreaming() {
    console.log('Executing with streaming...');

    updateState();

    const input = document.getElementById('user-input').value.trim();

    if (!input) {
        alert('Please enter an input prompt');
        return;
    }

    if (state.agents.length === 0) {
        alert('Please add at least one agent');
        return;
    }

    // Create WebSocket connection
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    console.log('Connecting to WebSocket:', wsUrl);

    const ws = new WebSocket(wsUrl);
    state.ws = ws;

    ws.onopen = () => {
        console.log('WebSocket connected');

        // Initialize visualization for pattern
        initializeStreamingVisualization(state.pattern);

        // Send execution request
        ws.send(JSON.stringify({
            agents: state.agents,
            pattern: state.patternOptions,
            input: input,
        }));
    };

    ws.onmessage = (event) => {
        const message = JSON.parse(event.data);
        console.log('WebSocket message:', message);
        handleStreamingEvent(message);
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        alert('WebSocket connection failed');
    };

    ws.onclose = () => {
        console.log('WebSocket closed');
        state.ws = null;
    };
}

async function compareAllPatterns() {
    console.log('Comparing all patterns...');

    updateState();

    const input = document.getElementById('user-input').value.trim();

    if (!input) {
        alert('Please enter an input prompt');
        return;
    }

    if (state.agents.length === 0) {
        alert('Please add at least one agent');
        return;
    }

    // Show loading in comparison grid
    const grid = document.getElementById('comparison-grid');
    grid.style.display = 'grid';
    grid.innerHTML = '<div class="loading-spinner"><div class="spinner"></div><p>Comparing all patterns...</p></div>';

    // Hide regular results
    document.getElementById('results-container').style.display = 'none';

    try {
        const response = await fetch('/api/compare', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                agents: state.agents,
                input: input,
            }),
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();
        console.log('Comparison results:', data);

        // Display comparison grid
        displayComparisonGrid(data.results);
    } catch (error) {
        console.error('Comparison failed:', error);
        alert(`Comparison failed: ${error.message}`);
        grid.style.display = 'none';
    }
}

// ========================================
// Visualization
// ========================================

function showVisualization(type) {
    const viz = document.getElementById('viz-content');
    viz.innerHTML = '';

    if (type === 'loading') {
        viz.innerHTML = '<div class="loading-spinner"><div class="spinner"></div><p>Executing pattern...</p></div>';
        return;
    }

    // Pattern-specific visualization will be added by streaming
}

function hideVisualization() {
    const viz = document.getElementById('viz-content');
    viz.innerHTML = '<p class="viz-placeholder">Click "Execute" to start visualization</p>';
}

function initializeStreamingVisualization(patternType) {
    const viz = document.getElementById('viz-content');
    viz.innerHTML = '';

    const container = document.createElement('div');
    container.className = `pattern-viz ${patternType}`;
    container.id = 'streaming-viz';

    switch (patternType) {
        case 'sequential':
            state.agents.forEach((agent, idx) => {
                const node = document.createElement('div');
                node.className = 'agent-node pending';
                node.id = `agent-${agent.id}`;
                node.textContent = agent.id;
                container.appendChild(node);

                if (idx < state.agents.length - 1) {
                    const arrow = document.createElement('div');
                    arrow.className = 'arrow';
                    arrow.textContent = '→';
                    container.appendChild(arrow);
                }
            });
            break;

        case 'concurrent':
            const concurrentContainer = document.createElement('div');
            concurrentContainer.className = 'concurrent-agents';

            state.agents.forEach(agent => {
                const node = document.createElement('div');
                node.className = 'agent-node pending';
                node.id = `agent-${agent.id}`;
                node.textContent = agent.id;
                concurrentContainer.appendChild(node);
            });

            container.appendChild(concurrentContainer);

            const aggregator = document.createElement('div');
            aggregator.className = 'aggregator';
            aggregator.textContent = '⇓ Aggregation ⇓';
            container.appendChild(aggregator);
            break;

        case 'group_chat':
            container.id = 'chat-container';
            break;

        case 'handoff':
            container.className = 'pattern-viz handoff';
            container.innerHTML = '<div class="handoff-chain" id="handoff-chain"></div>';
            break;

        case 'magentic':
            container.innerHTML = '<div class="ledger" id="task-ledger"></div>';
            break;
    }

    viz.appendChild(container);
}

function handleStreamingEvent(event) {
    console.log('Streaming event:', event.type);

    switch (event.type) {
        case 'agent_start':
            updateAgentState(event.agent_id, 'processing');
            break;

        case 'agent_complete':
            updateAgentState(event.agent_id, 'completed');
            break;

        case 'agent_error':
            updateAgentState(event.agent_id, 'error');
            break;

        case 'pattern_step':
            addPatternStep(event.message);
            break;

        case 'pattern_complete':
            handlePatternComplete(event);
            break;

        case 'pattern_error':
            alert(`Pattern execution failed: ${event.error}`);
            hideVisualization();
            break;
    }
}

function updateAgentState(agentId, state) {
    const node = document.getElementById(`agent-${agentId}`);
    if (node) {
        node.className = `agent-node ${state}`;
    }
}

function addPatternStep(message) {
    const pattern = state.pattern;

    if (pattern === 'group_chat') {
        const container = document.getElementById('chat-container');
        if (container) {
            const msg = document.createElement('div');
            msg.className = 'chat-message';
            msg.innerHTML = `<strong>${message.split(':')[0]}:</strong> ${message.split(':').slice(1).join(':')}`;
            container.appendChild(msg);
            container.scrollTop = container.scrollHeight;
        }
    } else if (pattern === 'handoff') {
        const chain = document.getElementById('handoff-chain');
        if (chain) {
            const step = document.createElement('div');
            step.className = 'handoff-arrow';
            step.textContent = message;
            chain.appendChild(step);
        }
    } else if (pattern === 'magentic') {
        const ledger = document.getElementById('task-ledger');
        if (ledger) {
            const task = document.createElement('div');
            if (message.startsWith('Completed:')) {
                task.className = 'task completed';
                task.textContent = '✓ ' + message.replace('Completed: ', '');
            } else if (message.startsWith('Working on:')) {
                task.className = 'task in-progress';
                task.textContent = '⟳ ' + message.replace('Working on: ', '');
            } else {
                task.className = 'task pending';
                task.textContent = '○ ' + message;
            }
            ledger.appendChild(task);
        }
    }
}

function handlePatternComplete(event) {
    console.log('Pattern execution complete');

    // Close WebSocket
    if (state.ws) {
        state.ws.close();
        state.ws = null;
    }

    // Display final results
    displayResults({
        output: event.output,
        execution_trace: [],
        metadata: event.metadata,
        pattern_name: state.pattern,
        duration_ms: 0,
    });
}

// ========================================
// Results Display
// ========================================

function displayResults(result) {
    const container = document.getElementById('results-container');
    const content = document.getElementById('results-content');

    container.style.display = 'block';
    content.innerHTML = '';

    // Metadata
    const metadata = document.createElement('div');
    metadata.className = 'result-metadata';
    metadata.innerHTML = `
        <span class="badge">${result.pattern_name}</span>
        <span class="metric">Duration: ${result.duration_ms}ms</span>
        <span class="metric">Agents: ${state.agents.length}</span>
    `;
    content.appendChild(metadata);

    // Output
    const output = document.createElement('div');
    output.className = 'result-output';
    output.innerHTML = `
        <h4>Final Output</h4>
        <pre>${escapeHtml(result.output)}</pre>
    `;
    content.appendChild(output);

    // Trace (if available)
    if (result.execution_trace && result.execution_trace.length > 0) {
        const details = document.createElement('details');
        details.innerHTML = `
            <summary>Execution Trace (${result.execution_trace.length} steps)</summary>
        `;

        const traceContainer = document.createElement('div');
        result.execution_trace.forEach((step, idx) => {
            const stepDiv = document.createElement('div');
            stepDiv.className = 'trace-step';
            stepDiv.innerHTML = `<strong>Step ${idx + 1}:</strong><pre>${escapeHtml(step)}</pre>`;
            traceContainer.appendChild(stepDiv);
        });

        details.appendChild(traceContainer);
        content.appendChild(details);
    }

    // Scroll to results
    container.scrollIntoView({ behavior: 'smooth', block: 'start' });
}

function displayComparisonGrid(results) {
    const grid = document.getElementById('comparison-grid');
    grid.innerHTML = '';

    Object.entries(results).forEach(([patternName, result]) => {
        const column = document.createElement('div');
        column.className = 'comparison-column';

        column.innerHTML = `
            <h3>${patternName}</h3>
            <div class="viz-mini">
                ${getPatternIcon(patternName)}
            </div>
            <div class="result-preview">${escapeHtml(result.output.substring(0, 200))}${result.output.length > 200 ? '...' : ''}</div>
            <div class="metrics">${result.duration_ms}ms | ${state.agents.length} agents</div>
        `;

        grid.appendChild(column);
    });
}

function getPatternIcon(patternName) {
    const lower = patternName.toLowerCase();
    if (lower.includes('sequential')) return '→→→';
    if (lower.includes('concurrent')) return '⇉';
    if (lower.includes('group')) return '💬';
    if (lower.includes('handoff')) return '↷';
    if (lower.includes('magentic')) return '☰';
    return '🔄';
}

// ========================================
// Preset Loading
// ========================================

function loadPreset(presetName) {
    const preset = state.presets.find(p => p.name === presetName);
    if (!preset) return;

    console.log('Loading preset:', presetName);

    // Clear existing agents
    document.getElementById('agents-container').innerHTML = '';

    // Add preset agents
    preset.agents.forEach(agent => {
        addAgent(agent.id, agent.provider, agent.model, agent.system_prompt);
    });

    // Set pattern
    const patternType = preset.pattern.type;
    document.getElementById('pattern-type').value = patternType;
    updatePatternOptions();

    // Set pattern-specific options
    if (patternType === 'concurrent' && preset.pattern.aggregation) {
        setTimeout(() => {
            const aggSelect = document.getElementById('aggregation');
            if (aggSelect) aggSelect.value = preset.pattern.aggregation;
        }, 100);
    } else if (patternType === 'group_chat' && preset.pattern.max_rounds) {
        setTimeout(() => {
            const roundsInput = document.getElementById('max-rounds');
            if (roundsInput) roundsInput.value = preset.pattern.max_rounds;
        }, 100);
    } else if (patternType === 'handoff' && preset.pattern.max_hops) {
        setTimeout(() => {
            const hopsInput = document.getElementById('max-hops');
            if (hopsInput) hopsInput.value = preset.pattern.max_hops;
        }, 100);
    } else if (patternType === 'magentic' && preset.pattern.max_iterations) {
        setTimeout(() => {
            const iterInput = document.getElementById('max-iterations');
            if (iterInput) iterInput.value = preset.pattern.max_iterations;
        }, 100);
    }

    // Set sample input
    if (preset.sample_input) {
        document.getElementById('user-input').value = preset.sample_input;
    }

    // Reset results
    document.getElementById('results-container').style.display = 'none';
    document.getElementById('comparison-grid').style.display = 'none';
    hideVisualization();

    updateState();
}

// ========================================
// Utilities
// ========================================

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
