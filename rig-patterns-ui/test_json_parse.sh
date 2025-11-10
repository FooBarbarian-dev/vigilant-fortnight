#!/bin/bash
# Test JSON parsing with the test file

cd "$(dirname "$0")"

echo "Testing JSON parsing with test_compare_request.json..."

cargo test test_parse_compare_request -- --nocapture --exact

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ JSON parsing test passed!"
    echo ""
    echo "Now you can start the server with:"
    echo "  cd rig-patterns-ui && cargo run --release"
    echo ""
    echo "Or use the top-level launch script:"
    echo "  ./run.sh"
else
    echo "❌ JSON parsing test failed"
    exit 1
fi
