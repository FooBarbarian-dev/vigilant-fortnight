@echo off
REM ========================================
REM rig-patterns Launch Script (Windows)
REM ========================================
REM This script:
REM 1. Pulls latest changes from git
REM 2. Builds everything in release mode
REM 3. Launches the pattern comparison UI
REM ========================================

setlocal enabledelayedexpansion

echo ========================================
echo RIG-PATTERNS LAUNCH SCRIPT
echo ========================================
echo.

REM ========================================
REM Step 1: Check prerequisites
REM ========================================
echo ========================================
echo Step 1: Checking Prerequisites
echo ========================================

REM Check if git is installed
where git >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] git is not installed. Please install git first.
    echo Visit: https://git-scm.com/download/win
    exit /b 1
)
echo [OK] git found

REM Check if cargo is installed
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] cargo is not installed. Please install Rust first.
    echo Visit: https://rustup.rs/
    exit /b 1
)
echo [OK] cargo found

REM Check for API keys
if "%OPENAI_API_KEY%"=="" if "%ANTHROPIC_API_KEY%"=="" if "%COHERE_API_KEY%"=="" (
    echo [ERROR] No LLM API keys found!
    echo.
    echo Please set at least one of the following environment variables:
    echo   set OPENAI_API_KEY=sk-...
    echo   set ANTHROPIC_API_KEY=sk-ant-...
    echo   set COHERE_API_KEY=...
    echo.
    echo Or add them to your system environment variables for persistence.
    exit /b 1
)

if not "%OPENAI_API_KEY%"=="" echo [OK] OPENAI_API_KEY is set
if not "%ANTHROPIC_API_KEY%"=="" echo [OK] ANTHROPIC_API_KEY is set
if not "%COHERE_API_KEY%"=="" echo [OK] COHERE_API_KEY is set
echo.

REM ========================================
REM Step 2: Pull latest changes
REM ========================================
echo ========================================
echo Step 2: Pulling Latest Changes
echo ========================================

REM Check if we're in a git repository
if not exist ".git" (
    echo [WARNING] Not a git repository. Skipping git pull.
) else (
    echo Fetching latest changes...

    REM Get current branch
    for /f "tokens=*" %%i in ('git rev-parse --abbrev-ref HEAD') do set CURRENT_BRANCH=%%i
    echo Current branch: !CURRENT_BRANCH!

    REM Pull latest changes
    git pull origin !CURRENT_BRANCH!
    if %ERRORLEVEL% NEQ 0 (
        echo [WARNING] Failed to pull changes. Continuing with local version...
    ) else (
        echo [OK] Successfully pulled latest changes
    )
)
echo.

REM ========================================
REM Step 3: Build rig-patterns library
REM ========================================
echo ========================================
echo Step 3: Building rig-patterns Library
echo ========================================

echo Building in release mode...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Failed to build rig-patterns library
    exit /b 1
)
echo [OK] rig-patterns library built successfully
echo.

REM ========================================
REM Step 4: Build rig-patterns-ui
REM ========================================
echo ========================================
echo Step 4: Building rig-patterns-ui
echo ========================================

cd rig-patterns-ui

echo Building UI in release mode...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Failed to build rig-patterns-ui
    exit /b 1
)
echo [OK] rig-patterns-ui built successfully
echo.

REM ========================================
REM Step 5: Launch UI
REM ========================================
echo ========================================
echo Step 5: Launching Pattern Comparison UI
echo ========================================

echo [OK] Build complete! Starting server...
echo.
echo ============================================
echo   RIG-PATTERNS PATTERN COMPARISON UI
echo ============================================
echo.
echo Server will start at: http://localhost:3009
echo.
echo To use the UI:
echo   1. Open http://localhost:3009 in your browser
echo   2. Configure agents in each pattern tab
echo   3. Enter a root prompt
echo   4. Click 'Execute All Patterns'
echo   5. Watch the tabs for real-time execution
echo.
echo To use a different port: set PORT=8080 ^&^& run.bat
echo Press Ctrl+C to stop the server
echo.

REM Run the server
cargo run --release
