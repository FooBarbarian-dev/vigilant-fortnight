#!/bin/bash

# ========================================
# rig-patterns Launch Script
# ========================================
# This script:
# 1. Pulls latest changes from git
# 2. Builds everything in release mode
# 3. Launches the pattern comparison UI
# ========================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print with color
print_header() {
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}→ $1${NC}"
}

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

print_header "RIG-PATTERNS LAUNCH SCRIPT"

# ========================================
# Step 1: Check prerequisites
# ========================================
print_header "Step 1: Checking Prerequisites"

# Check if git is installed
if ! command -v git &> /dev/null; then
    print_error "git is not installed. Please install git first."
    exit 1
fi
print_success "git found"

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    print_error "cargo is not installed. Please install Rust first."
    echo "Visit: https://rustup.rs/"
    exit 1
fi
print_success "cargo found"

# Check for API keys
if [ -z "$OPENAI_API_KEY" ] && [ -z "$ANTHROPIC_API_KEY" ] && [ -z "$COHERE_API_KEY" ]; then
    print_error "No LLM API keys found!"
    echo ""
    echo "Please set at least one of the following environment variables:"
    echo "  export OPENAI_API_KEY=\"sk-...\""
    echo "  export ANTHROPIC_API_KEY=\"sk-ant-...\""
    echo "  export COHERE_API_KEY=\"...\""
    echo ""
    echo "You can add these to your ~/.bashrc or ~/.zshrc for persistence."
    exit 1
fi

if [ -n "$OPENAI_API_KEY" ]; then
    print_success "OPENAI_API_KEY is set"
fi
if [ -n "$ANTHROPIC_API_KEY" ]; then
    print_success "ANTHROPIC_API_KEY is set"
fi
if [ -n "$COHERE_API_KEY" ]; then
    print_success "COHERE_API_KEY is set"
fi

# ========================================
# Step 2: Pull latest changes
# ========================================
print_header "Step 2: Pulling Latest Changes"

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    print_error "Not a git repository. Skipping git pull."
else
    print_info "Fetching latest changes..."

    # Get current branch
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
    print_info "Current branch: $CURRENT_BRANCH"

    # Stash any local changes
    if ! git diff-index --quiet HEAD --; then
        print_info "Stashing local changes..."
        git stash push -m "Auto-stash before pull at $(date)"
    fi

    # Pull latest changes
    if git pull origin "$CURRENT_BRANCH"; then
        print_success "Successfully pulled latest changes"
    else
        print_error "Failed to pull changes. Continuing with local version..."
    fi

    # Apply stash if we created one
    if git stash list | grep -q "Auto-stash before pull"; then
        print_info "Applying stashed changes..."
        git stash pop || print_error "Could not apply stashed changes"
    fi
fi

# ========================================
# Step 3: Build rig-patterns library
# ========================================
print_header "Step 3: Building rig-patterns Library"

print_info "Building in release mode..."
if cargo build --release; then
    print_success "rig-patterns library built successfully"
else
    print_error "Failed to build rig-patterns library"
    exit 1
fi

# ========================================
# Step 4: Build rig-patterns-ui
# ========================================
print_header "Step 4: Building rig-patterns-ui"

cd rig-patterns-ui

print_info "Building UI in release mode..."
if cargo build --release; then
    print_success "rig-patterns-ui built successfully"
else
    print_error "Failed to build rig-patterns-ui"
    exit 1
fi

# ========================================
# Step 5: Launch UI
# ========================================
print_header "Step 5: Launching Pattern Comparison UI"

print_success "Build complete! Starting server..."
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  RIG-PATTERNS PATTERN COMPARISON UI    ${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${CYAN}Server will start at: ${GREEN}http://localhost:3000${NC}"
echo ""
echo -e "${YELLOW}To use the UI:${NC}"
echo "  1. Open http://localhost:3000 in your browser"
echo "  2. Configure agents in each pattern tab"
echo "  3. Enter a root prompt"
echo "  4. Click '⚡ EXECUTE ALL PATTERNS'"
echo "  5. Watch the tabs for real-time execution"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
echo ""

# Run the server
exec cargo run --release
