#!/bin/bash
# Hardware Candlestick Chart - Build and Run Helper Script
# =========================================================
# Usage:
#   ./build_and_run.sh            # Build release version and run
#   ./build_and_run.sh debug      # Debug build with verbose output
#   ./build_and_run.sh test 10    # Build, run for 10 seconds as demo

set -e  # Exit on error

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN_NAME="candlestick"

show_help() {
    echo "Hardware Candlestick Chart Builder/Runner"
    echo ""
    echo "Usage: $0 [mode] [duration]"
    echo ""
    echo "Modes:"
    echo "  (empty)   - Build release version and run (default)"
    echo "  debug     - Build debug version with verbose output"
    echo "  test      - Build and run for demo duration"
    echo ""
    echo "Arguments:"
    echo "  duration  - Optional seconds to run before exit (for testing)"
    echo ""
    echo "Examples:"
    echo "  $0                     # Release build + run"
    echo "  $0 debug               # Debug build + run"
    echo "  $0 test 60             # Run demo for 60 seconds"
    echo "  $0 release             # Build optimized release version"
    echo ""
}

check_rust() {
    if ! command -v cargo &> /dev/null; then
        echo "❌ Rust/Cargo not found. Please install Rust first:"
        echo "   curl --proto '=https' --tlsv1.2 -LsSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    if ! command -v rustc &> /dev/null; then
        echo "❌ Rust compiler not found."
        exit 1
    fi
    
    echo "✓ Rust is installed"
}

check_dependencies() {
    # Check if we can build the project
    cd "$PROJECT_DIR"
    cargo check --quiet 2>/dev/null || {
        echo "⚠️  Dependencies will be downloaded on first run (cargo install)"
    }
}

build_project() {
    local mode="${1:-release}"
    
    if [ "$mode" == "debug" ]; then
        cargo build --bin $BIN_NAME
        echo "✓ Debug build complete: target/debug/${BIN_NAME}"
    elif [ "$mode" == "release" ]; then
        cargo build --release --bin $BIN_NAME
        echo "✓ Release build complete: target/release/${BIN_NAME}"
    else
        cargo build --bin $BIN_NAME
        echo "✓ Default build complete: target/debug/${BIN_NAME}"
    fi
}

run_application() {
    local mode="${1:-}"
    local duration="${2:-}"
    
    # Determine which binary to use
    if [ "$mode" == "debug" ]; then
        BIN_PATH="./target/debug/$BIN_NAME"
    else
        BIN_PATH="./target/release/$BIN_NAME"
    fi
    
    # Add duration flag if specified and mode is not debug
    ARGS=""
    if [ -n "$duration" ] && [ "$mode" != "debug" ]; then
        ARGS="--duration $duration"
        echo "Running for ${duration} seconds demo..."
    elif [ "$mode" == "test" ]; then
        # If mode is 'test', duration is the first arg
        ARGS="--duration $duration"
        echo "Running test mode for ${duration} seconds..."
    fi
    
    echo ""
    echo "🕯️ Starting Hardware Candlestick Chart..."
    echo ""
    "$BIN_PATH" $ARGS
}

# Main execution logic
main() {
    local mode="${1:-}"
    local duration="${2:-}"
    
    show_help
    
    check_rust
    build_project "$mode"
    run_application "$mode" "$duration"
}

# If script is being sourced, show help; otherwise execute
if [ "${BASH_SOURCE[0]}" == "${0}" ]; then
    main "$@"
fi
