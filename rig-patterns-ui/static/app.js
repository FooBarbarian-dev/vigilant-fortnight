// ========================================
// rig-patterns Pattern Comparison UI
// ========================================

// State management
let state = {
    patterns: {
        sequential: [],
        concurrent: [],
        group_chat: [],
        handoff: [],
        magentic: []
    },
    currentPattern: 'sequential',
    ws: null,
    isExecuting: false
};

// ========================================
// Initialization
// ========================================

document.addEventListener('DOMContentLoaded', () => {
    console.log('Initializing rig-patterns Pattern Comparison UI...');

    // Initialize Mermaid
    mermaid.initialize({
        startOnLoad: false,  // Changed to false - we'll render manually
        theme: 'dark',
        themeVariables: {
            primaryColor: '#16213e',
            primaryTextColor: '#00ffff',
            primaryBorderColor: '#00ffff',
            lineColor: '#00ff00',
            secondaryColor: '#1a1a2e',
            tertiaryColor: '#0f3460'
        }
    });

    // Initialize default agents for each pattern
    initializeDefaultAgents();

    // Set up event listeners
    setupEventListeners();

    // Render initial DAG only for active panel
    renderDAG('sequential');

    console.log('✅ Initialization complete!');
});

// ========================================
// Default Agents Setup
// ========================================

function initializeDefaultAgents() {
    // Sequential: 2 agents in chain
    addAgent('sequential', 'agent1', 'openai', 'gpt-5',
        'You are an Information Gatherer. Your role is to receive a user query, research and gather comprehensive information about the topic, and provide a detailed summary. Focus on extracting key facts, concepts, and context that will be useful for further analysis. Present your findings in a clear, structured format.');
    addAgent('sequential', 'agent2', 'openai', 'gpt-5',
        'You are an Expert Analyzer. You receive the gathered information from the previous agent and perform deep analysis. Your role is to synthesize insights, identify patterns, draw conclusions, and provide actionable recommendations. Build upon the information provided and add expert-level interpretation.');

    // Concurrent: 2 agents in parallel
    addAgent('concurrent', 'reviewer1', 'openai', 'gpt-5',
        'You are a Technical Reviewer. Evaluate the input from a technical perspective, focusing on accuracy, feasibility, technical requirements, and potential implementation challenges. Assess technical risks, performance implications, and best practices. Provide specific technical recommendations.');
    addAgent('concurrent', 'reviewer2', 'openai', 'gpt-5',
        'You are a Business Analyst. Review the input from a business perspective, considering market viability, user needs, cost-benefit analysis, and strategic alignment. Evaluate business impact, ROI, and competitive advantages. Provide business-focused recommendations and insights.');

    // Group Chat: 2 agents discussing
    addAgent('group_chat', 'writer', 'openai', 'gpt-5',
        'You are a Content Writer. Your role is to create clear, engaging, and well-structured content based on the user\'s request. Draft initial versions and respond to feedback constructively. Wait for the editor\'s review and suggestions before declaring CONSENSUS_REACHED. Be open to revisions and improvements.');
    addAgent('group_chat', 'editor', 'openai', 'gpt-5',
        'You are a Content Editor. Review the writer\'s work for clarity, coherence, grammar, structure, and overall quality. Provide constructive feedback on improvements needed. When you believe the content meets high standards and no further revisions are required, include CONSENSUS_REACHED in your final response to conclude the discussion.');

    // Handoff: 2 agents with routing
    addAgent('handoff', 'triage', 'openai', 'gpt-5',
        'You are a Triage Coordinator. Your role is to analyze incoming requests, assess their complexity and requirements, and decide whether you can handle them directly or need to route them to a specialist. For simple queries, provide a direct response. For complex or specialized tasks, use HANDOFF:specialist to transfer the request. Explain your routing decisions clearly.');
    addAgent('handoff', 'specialist', 'openai', 'gpt-5',
        'You are a Domain Specialist. You handle complex, specialized tasks that have been handed off to you by the triage coordinator. Leverage your deep expertise to provide comprehensive, expert-level responses. Address the specific challenges and requirements that warranted the handoff. Provide detailed, authoritative answers.');

    // Magentic: Manager + Worker
    addAgent('magentic', 'manager', 'openai', 'gpt-5',
        'You are a Project Manager. Your role is to receive complex requests and break them down into 2-4 specific, actionable subtasks. Each subtask should be clear, focused, and independently executable. Start each subtask with "- " on a new line. Think strategically about task decomposition, dependencies, and prioritization. Make subtasks concrete and measurable.');
    addAgent('magentic', 'worker', 'openai', 'gpt-5',
        'You are a Task Worker. You receive individual subtasks from the manager and execute them thoroughly. Focus on completing your assigned task with high quality and attention to detail. Provide clear, complete results for your specific subtask. Work independently and deliver concrete outputs that can be integrated into the final solution.');
}

// ========================================
// Event Listeners
// ========================================

function setupEventListeners() {
    // Pattern tab switching
    document.querySelectorAll('.pattern-tab').forEach(tab => {
        tab.addEventListener('click', () => {
            const pattern = tab.dataset.pattern;
            switchToPattern(pattern);
        });
    });

    // Execute All button
    document.getElementById('execute-all').addEventListener('click', executeAllPatterns);

    // Add agent buttons
    document.querySelectorAll('.add-agent-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const pattern = btn.dataset.pattern;
            const agentNum = state.patterns[pattern].length + 1;
            addAgent(pattern, `agent${agentNum}`, 'openai', 'gpt-5', 'You are a specialized AI assistant. Your role is to process inputs according to your specific responsibilities in this pattern. Provide clear, accurate, and helpful responses that contribute to the overall workflow.');
            renderDAG(pattern);
        });
    });
}

// ========================================
// Pattern Tab Switching
// ========================================

function switchToPattern(pattern) {
    state.currentPattern = pattern;

    // Update tabs
    document.querySelectorAll('.pattern-tab').forEach(tab => {
        tab.classList.remove('active');
        if (tab.dataset.pattern === pattern) {
            tab.classList.add('active');
        }
    });

    // Update panels
    document.querySelectorAll('.pattern-panel').forEach(panel => {
        panel.classList.remove('active');
        if (panel.id === `panel-${pattern}`) {
            panel.classList.add('active');
        }
    });

    // Render DAG for the newly active pattern
    renderDAG(pattern);

    console.log(`Switched to ${pattern} pattern`);
}

// ========================================
// Agent Management
// ========================================

function addAgent(pattern, id, provider, model, prompt) {
    const agent = { id, provider, model, system_prompt: prompt };
    state.patterns[pattern].push(agent);

    // Add agent card to UI
    const container = document.getElementById(`agents-${pattern}`);
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

    // Update state on changes
    const updateAgent = () => {
        const index = state.patterns[pattern].findIndex(a => a.id === idInput.value || a.id === id);
        if (index !== -1) {
            state.patterns[pattern][index] = {
                id: idInput.value,
                provider: providerSelect.value,
                model: modelInput.value,
                system_prompt: promptTextarea.value
            };
            renderDAG(pattern);
        }
    };

    // Auto-update model when provider changes
    providerSelect.addEventListener('change', () => {
        const provider = providerSelect.value;
        const currentModel = modelInput.value;

        // Only auto-update if current model doesn't match provider
        if (provider === 'anthropic' && currentModel.startsWith('gpt-')) {
            modelInput.value = 'claude-sonnet-4-5-20250929';
        } else if (provider === 'openai' && currentModel.startsWith('claude-')) {
            modelInput.value = 'gpt-5';
        } else if (provider === 'cohere' && !currentModel.startsWith('command-')) {
            modelInput.value = 'command-r-plus';
        }

        updateAgent();
    });

    idInput.addEventListener('input', updateAgent);
    modelInput.addEventListener('input', updateAgent);
    promptTextarea.addEventListener('input', updateAgent);

    // Remove button
    removeBtn.addEventListener('click', () => {
        const index = state.patterns[pattern].findIndex(a => a.id === idInput.value);
        if (index !== -1) {
            state.patterns[pattern].splice(index, 1);
            card.remove();
            renderDAG(pattern);
        }
    });

    container.appendChild(clone);

    // Update DAG to reflect new agent
    renderDAG(pattern);
}

// ========================================
// DAG Visualization
// ========================================

function renderAllDAGs() {
    Object.keys(state.patterns).forEach(pattern => {
        renderDAG(pattern);
    });
}

function renderDAG(pattern) {
    const agents = state.patterns[pattern];
    const dagEl = document.getElementById(`dag-${pattern}`);

    let mermaidCode = '';

    switch (pattern) {
        case 'sequential':
            mermaidCode = generateSequentialDAG(agents);
            break;
        case 'concurrent':
            mermaidCode = generateConcurrentDAG(agents);
            break;
        case 'group_chat':
            mermaidCode = generateGroupChatDAG(agents);
            break;
        case 'handoff':
            mermaidCode = generateHandoffDAG(agents);
            break;
        case 'magentic':
            mermaidCode = generateMagenticDAG(agents);
            break;
    }

    dagEl.textContent = mermaidCode;
    dagEl.removeAttribute('data-processed');
    mermaid.run({ nodes: [dagEl] });
}

function generateSequentialDAG(agents) {
    let dag = 'graph LR\n    INPUT[Input]';
    agents.forEach((agent, i) => {
        const nodeId = `A${i + 1}`;
        dag += ` --> ${nodeId}[${agent.id}<br/>${agent.provider}]`;
    });
    dag += ' --> OUTPUT[Output]\n';
    dag += '    style INPUT fill:#1a1a2e,stroke:#00ffff\n';
    agents.forEach((agent, i) => {
        dag += `    style A${i + 1} fill:#16213e,stroke:#00ff00\n`;
    });
    dag += '    style OUTPUT fill:#1a1a2e,stroke:#ff00ff';
    return dag;
}

function generateConcurrentDAG(agents) {
    let dag = 'graph TD\n    INPUT[Input]';
    agents.forEach((agent, i) => {
        const nodeId = `A${i + 1}`;
        dag += `\n    INPUT --> ${nodeId}[${agent.id}<br/>${agent.provider}]`;
    });
    dag += '\n    ';
    agents.forEach((agent, i) => {
        dag += `A${i + 1} --> `;
    });
    dag += 'AGG[Aggregator]\n    AGG --> OUTPUT[Output]\n';
    dag += '    style INPUT fill:#1a1a2e,stroke:#00ffff\n';
    agents.forEach((agent, i) => {
        dag += `    style A${i + 1} fill:#16213e,stroke:#00ff00\n`;
    });
    dag += '    style AGG fill:#16213e,stroke:#ffff00\n';
    dag += '    style OUTPUT fill:#1a1a2e,stroke:#ff00ff';
    return dag;
}

function generateGroupChatDAG(agents) {
    let dag = 'graph TD\n    INPUT[Input] --> R1[Round 1]\n';
    agents.forEach((agent, i) => {
        dag += `    R1 --> A1_${i}[${agent.id}<br/>${agent.provider}]\n`;
    });
    agents.forEach((agent, i) => {
        dag += `    A1_${i} --> `;
    });
    dag += 'CON{Consensus?}\n';
    dag += '    CON -->|Yes| OUTPUT[Output]\n';
    dag += '    CON -->|No| R2[Round 2...]\n';
    dag += '    style INPUT fill:#1a1a2e,stroke:#00ffff\n';
    dag += '    style OUTPUT fill:#1a1a2e,stroke:#ff00ff';
    return dag;
}

function generateHandoffDAG(agents) {
    let dag = 'graph LR\n    INPUT[Input]';
    agents.forEach((agent, i) => {
        const nodeId = `A${i + 1}`;
        if (i === 0) {
            dag += ` --> ${nodeId}[${agent.id}<br/>${agent.provider}]`;
        } else {
            dag += `\n    A${i} -->|HANDOFF?| ${nodeId}[${agent.id}<br/>${agent.provider}]`;
        }
    });
    dag += ' --> OUTPUT[Output]\n';
    dag += '    style INPUT fill:#1a1a2e,stroke:#00ffff\n';
    agents.forEach((agent, i) => {
        dag += `    style A${i + 1} fill:#16213e,stroke:#00ff00\n`;
    });
    dag += '    style OUTPUT fill:#1a1a2e,stroke:#ff00ff';
    return dag;
}

function generateMagenticDAG(agents) {
    if (agents.length === 0) return 'graph TD\n    EMPTY[No Agents]';

    const manager = agents[0];
    const workers = agents.slice(1);

    let dag = `graph TD\n    INPUT[Input] --> MGR[${manager.id}<br/>${manager.provider}]\n`;
    workers.forEach((worker, i) => {
        dag += `    MGR --> T${i + 1}[Task ${i + 1}]\n`;
        dag += `    T${i + 1} --> W${i + 1}[${worker.id}<br/>${worker.provider}]\n`;
    });
    if (workers.length > 0) {
        dag += '    ';
        workers.forEach((worker, i) => {
            dag += `W${i + 1} --> `;
        });
        dag += 'SYN[Synthesize]\n    SYN --> MGR2[Manager]\n    MGR2 --> OUTPUT[Output]\n';
    } else {
        dag += '    MGR --> OUTPUT[Output]\n';
    }
    dag += '    style INPUT fill:#1a1a2e,stroke:#00ffff\n';
    dag += `    style MGR fill:#16213e,stroke:#ffff00\n`;
    workers.forEach((worker, i) => {
        dag += `    style W${i + 1} fill:#16213e,stroke:#00ff00\n`;
    });
    dag += '    style OUTPUT fill:#1a1a2e,stroke:#ff00ff';
    return dag;
}

// ========================================
// Pattern Execution
// ========================================

function executeAllPatterns() {
    const rootPrompt = document.getElementById('root-prompt').value.trim();

    if (!rootPrompt) {
        alert('Please enter a root prompt');
        return;
    }

    // Validate that all patterns have agents
    for (const pattern in state.patterns) {
        if (state.patterns[pattern].length === 0) {
            alert(`${pattern} pattern has no agents. Please add at least one agent to each pattern.`);
            return;
        }
    }

    if (state.isExecuting) {
        alert('Execution already in progress');
        return;
    }

    console.log('========== EXECUTING ALL PATTERNS ==========');
    console.log('Root Prompt:', rootPrompt);

    // Clear all logs and reset result cards
    Object.keys(state.patterns).forEach(pattern => {
        // Clear execution log
        const log = document.getElementById(`log-${pattern}`);
        log.innerHTML = '';

        // Update tab status to running
        updatePatternStatus(pattern, 'running');

        // Reset result card to running state
        const resultBody = document.getElementById(`result-${pattern}`);
        if (resultBody) {
            resultBody.innerHTML = '<div class="result-placeholder">Executing...</div>';
        }

        const resultStatus = document.getElementById(`status-result-${pattern}`);
        if (resultStatus) {
            resultStatus.className = 'result-status running';
            resultStatus.textContent = 'RUNNING';
        }
    });

    // Build pattern configs
    const patternConfigs = {};

    patternConfigs.sequential = {
        pattern: { type: 'sequential' },
        agents: state.patterns.sequential
    };

    patternConfigs.concurrent = {
        pattern: { type: 'concurrent', aggregation: 'combine' },
        agents: state.patterns.concurrent
    };

    patternConfigs.group_chat = {
        pattern: { type: 'group_chat', max_rounds: 3 },
        agents: state.patterns.group_chat
    };

    patternConfigs.handoff = {
        pattern: { type: 'handoff', max_hops: 5 },
        agents: state.patterns.handoff
    };

    patternConfigs.magentic = {
        pattern: { type: 'magentic', max_iterations: 5 },
        agents: state.patterns.magentic
    };

    const request = {
        input: rootPrompt,
        pattern_configs: patternConfigs
    };

    console.log('📤 Sending CompareRequest:', request);
    const requestJson = JSON.stringify(request);
    console.log('📦 JSON length:', requestJson.length, 'bytes');
    console.log('📦 JSON preview:', requestJson.substring(0, 500));

    // Connect WebSocket
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    const ws = new WebSocket(wsUrl);
    state.ws = ws;
    state.isExecuting = true;

    ws.onopen = () => {
        console.log('✅ WebSocket connected');
        updateWebSocketStatus('connected');
        console.log('📤 Sending JSON to server...');
        ws.send(requestJson);
    };

    ws.onmessage = (event) => {
        const message = JSON.parse(event.data);
        console.log(`📥 [${message.pattern_id}] ${message.type}`, message);
        handleExecutionEvent(message);
    };

    ws.onerror = (error) => {
        console.error('❌ WebSocket error:', error);
        updateWebSocketStatus('error');
        state.isExecuting = false;
    };

    ws.onclose = () => {
        console.log('🔌 WebSocket closed');
        console.log('========== EXECUTION COMPLETE ==========');
        updateWebSocketStatus('disconnected');
        state.isExecuting = false;
        state.ws = null;
    };
}

// ========================================
// Event Handling
// ========================================

function handleExecutionEvent(event) {
    const patternId = event.pattern_id;
    const log = document.getElementById(`log-${patternId}`);

    if (!log) {
        console.warn(`No log element found for pattern: ${patternId}`);
        return;
    }

    const timestamp = new Date(event.timestamp).toLocaleTimeString();
    let logEntry = '';

    switch (event.type) {
        case 'agent_receives_input':
            logEntry = `<div class="log-entry input">
                <span class="log-time">${timestamp}</span>
                <span class="log-agent">${event.agent_id} (${event.provider})</span>
                <span class="log-type">RECEIVES INPUT</span>
                <div class="log-content">${escapeHtml(event.input.substring(0, 100))}...</div>
            </div>`;
            break;

        case 'agent_thinking':
            logEntry = `<div class="log-entry thinking">
                <span class="log-time">${timestamp}</span>
                <span class="log-agent">${event.agent_id} (${event.provider})</span>
                <span class="log-type">THINKING...</span>
            </div>`;
            break;

        case 'agent_responds':
            logEntry = `<div class="log-entry response">
                <span class="log-time">${timestamp}</span>
                <span class="log-agent">${event.agent_id} (${event.provider})</span>
                <span class="log-type">RESPONDS</span>
                <div class="log-content">${escapeHtml(event.response)}</div>
            </div>`;
            break;

        case 'agent_handoff':
            logEntry = `<div class="log-entry handoff">
                <span class="log-time">${timestamp}</span>
                <span class="log-type">HANDOFF</span>
                <div class="log-content">${event.from_agent} (${event.from_provider}) → ${event.to_agent} (${event.to_provider})</div>
            </div>`;
            break;

        case 'pattern_step':
            logEntry = `<div class="log-entry step">
                <span class="log-time">${timestamp}</span>
                <span class="log-type">STEP</span>
                <div class="log-content">${escapeHtml(event.message)}</div>
            </div>`;
            break;

        case 'conversation_message':
            logEntry = `<div class="log-entry conversation ${event.message_type}">
                <span class="log-time">${timestamp}</span>
                <span class="log-agent">${event.from} (${event.provider})</span>
                <span class="log-type">${event.message_type.toUpperCase()}</span>
                <div class="log-content">${escapeHtml(event.message)}</div>
            </div>`;
            break;

        case 'pattern_complete':
            logEntry = `<div class="log-entry complete">
                <span class="log-time">${timestamp}</span>
                <span class="log-type">✅ PATTERN COMPLETE</span>
                <div class="log-content"><strong>Output:</strong> ${escapeHtml(event.output)}</div>
            </div>`;
            updatePatternStatus(patternId, 'completed');
            updateFinalResult(patternId, event.output, 'success');
            break;

        case 'pattern_error':
            logEntry = `<div class="log-entry error">
                <span class="log-time">${timestamp}</span>
                <span class="log-type">❌ ERROR</span>
                <div class="log-content">${escapeHtml(event.error)}</div>
            </div>`;
            updatePatternStatus(patternId, 'error');
            updateFinalResult(patternId, event.error, 'error');
            break;
    }

    if (logEntry) {
        log.insertAdjacentHTML('beforeend', logEntry);
        log.scrollTop = log.scrollHeight;
    }
}

// ========================================
// Status Updates
// ========================================

function updatePatternStatus(pattern, status) {
    const badge = document.getElementById(`status-${pattern}`);
    if (!badge) return;

    badge.className = 'status-badge';

    switch (status) {
        case 'running':
            badge.classList.add('running');
            badge.textContent = 'RUNNING';
            break;
        case 'completed':
            badge.classList.add('completed');
            badge.textContent = 'COMPLETE';
            break;
        case 'error':
            badge.classList.add('error');
            badge.textContent = 'ERROR';
            break;
        default:
            badge.textContent = 'READY';
    }
}

function updateFinalResult(patternId, content, resultType) {
    // Update result card body
    const resultContainer = document.getElementById(`result-${patternId}`);
    if (!resultContainer) {
        console.warn(`No result container found for pattern: ${patternId}`);
        return;
    }

    // Clear placeholder if present
    resultContainer.innerHTML = '';

    // Create result content div
    const resultDiv = document.createElement('div');
    resultDiv.className = `result-content ${resultType}`;
    resultDiv.textContent = content;

    resultContainer.appendChild(resultDiv);

    // Scroll to the result
    resultContainer.scrollTop = 0;

    // Update result card status badge
    const statusBadge = document.getElementById(`status-result-${patternId}`);
    if (statusBadge) {
        statusBadge.className = 'result-status';
        if (resultType === 'success') {
            statusBadge.classList.add('complete');
            statusBadge.textContent = 'COMPLETE';
        } else if (resultType === 'error') {
            statusBadge.classList.add('error');
            statusBadge.textContent = 'ERROR';
        }
    }

    // Scroll the comparison section into view
    const comparisonSection = document.querySelector('.results-comparison-section');
    if (comparisonSection) {
        comparisonSection.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
}

function updateWebSocketStatus(status) {
    const statusDot = document.getElementById('ws-status');
    const statusText = document.getElementById('ws-text');

    if (!statusDot || !statusText) return;

    statusDot.className = 'status-dot';
    switch (status) {
        case 'connected':
            statusDot.classList.add('active');
            statusText.textContent = 'CONNECTED';
            break;
        case 'disconnected':
            statusText.textContent = 'DISCONNECTED';
            break;
        case 'error':
            statusText.textContent = 'ERROR';
            break;
    }
}

// ========================================
// Utility Functions
// ========================================

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
