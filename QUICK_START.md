# Quick Start Guide

## Prerequisites

1. **Rust** - Install from https://rustup.rs/
2. **Git** - Install from https://git-scm.com/
3. **LLM API Keys** - At least one of:
   - OpenAI: https://platform.openai.com/api-keys
   - Anthropic: https://console.anthropic.com/
   - Cohere: https://dashboard.cohere.ai/

## 🚀 One-Command Launch

### Linux/macOS

```bash
# Set your API key(s) first
export OPENAI_API_KEY="sk-..."
# and/or
export ANTHROPIC_API_KEY="sk-ant-..."
# and/or
export COHERE_API_KEY="..."

# Run the launch script
./run.sh
```

### Windows

```cmd
REM Set your API key(s) first
set OPENAI_API_KEY=sk-...
REM and/or
set ANTHROPIC_API_KEY=sk-ant-...
REM and/or
set COHERE_API_KEY=...

REM Run the launch script
run.bat
```

## What the Script Does

1. ✅ **Checks Prerequisites** - Verifies git, cargo, and API keys
2. ✅ **Pulls Latest** - Gets the latest code from git
3. ✅ **Builds Library** - Compiles `rig-patterns` in release mode
4. ✅ **Builds UI** - Compiles `rig-patterns-ui` in release mode
5. ✅ **Launches Server** - Starts the web server at http://localhost:3009

## Using the UI

Once the server starts:

1. **Open Browser** → http://localhost:3009
2. **See 5 Tabs** → Sequential, Concurrent, Group Chat, Handoff, Magentic
3. **View DAGs** → Each tab shows the expected execution flow
4. **Configure Agents** → Edit agents per pattern (or use defaults)
5. **Enter Prompt** → Type your root prompt at the top
6. **Click Execute** → "⚡ EXECUTE ALL PATTERNS" runs everything in parallel
7. **Watch Tabs** → Switch tabs to see real-time execution logs

## Setting API Keys Permanently

### Linux/macOS

Add to `~/.bashrc` or `~/.zshrc`:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export COHERE_API_KEY="..."
```

Then reload: `source ~/.bashrc`

### Windows

**Option 1: System Environment Variables**
1. Right-click "This PC" → Properties
2. Advanced system settings → Environment Variables
3. Add new user variables:
   - `OPENAI_API_KEY` = `sk-...`
   - `ANTHROPIC_API_KEY` = `sk-ant-...`
   - `COHERE_API_KEY` = `...`

**Option 2: PowerShell Profile**

```powershell
# Edit profile
notepad $PROFILE

# Add these lines:
$env:OPENAI_API_KEY = "sk-..."
$env:ANTHROPIC_API_KEY = "sk-ant-..."
$env:COHERE_API_KEY = "..."
```

## Manual Build (Without Script)

If you prefer to build manually:

```bash
# 1. Pull latest
git pull

# 2. Build library
cargo build --release

# 3. Build and run UI
cd rig-patterns-ui
cargo run --release
```

## Troubleshooting

### Script Error: "Permission denied"

**Linux/macOS:**
```bash
chmod +x run.sh
```

### Script Error: "git not found"

Install git:
- Linux: `sudo apt-get install git` or `sudo yum install git`
- macOS: `brew install git` or install Xcode Command Line Tools
- Windows: Download from https://git-scm.com/download/win

### Script Error: "cargo not found"

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Or visit: https://rustup.rs/

### Script Error: "No LLM API keys found"

You need at least one API key. Set it before running:

```bash
export OPENAI_API_KEY="sk-..."  # Linux/macOS
set OPENAI_API_KEY=sk-...       # Windows
```

### Build Error: "linker not found"

**Linux:**
```bash
sudo apt-get install build-essential
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
Install Visual Studio Build Tools from:
https://visualstudio.microsoft.com/downloads/

### Browser Error: "Cannot connect to localhost:3009"

1. Check if server is running (should see logs in terminal)
2. Try: http://127.0.0.1:3009 instead
3. Check firewall isn't blocking port 3009
4. Make sure no other service is using port 3009

### WebSocket Error: "Connection failed"

1. Refresh the browser page
2. Check server logs for errors
3. Ensure API keys are valid
4. Try a different browser

## Next Steps

- See **PATTERN_COMPARISON_UI_GUIDE.md** for detailed UI usage
- See **FRONTEND_ROADMAP.md** for technical details
- See **examples/** directory for sample configurations

## Stop the Server

Press `Ctrl+C` in the terminal where the server is running.

## Update to Latest Version

Just run the script again - it will pull latest changes:

```bash
./run.sh          # Linux/macOS
run.bat           # Windows
```

## Development Mode

For faster builds during development:

```bash
cd rig-patterns-ui
cargo run  # Without --release flag
```

## Questions?

Check the documentation:
- **PATTERN_COMPARISON_UI_GUIDE.md** - UI usage guide
- **FRONTEND_ROADMAP.md** - Technical architecture
- **README.md** - Project overview

Or check the code:
- `src/` - rig-patterns library
- `rig-patterns-ui/src/` - UI backend
- `rig-patterns-ui/static/` - UI frontend
