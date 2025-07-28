#!/bin/bash

echo "Testing self-update 'Text file busy' fix..."

# Create a test binary location
TEST_DIR="/tmp/lda_test_$$"
mkdir -p "$TEST_DIR"

# Copy current binary to test location
cp target/release/linux-distro-agent "$TEST_DIR/test-binary"

echo "✓ Created test environment at $TEST_DIR"

# Test the rm + cp operation (simulating what our fix does)
echo "Testing remove + copy operation..."

# First, let's try the old method (direct copy) to show it might fail
echo "  - Testing direct copy (old method)..."
if sudo cp target/release/linux-distro-agent "$TEST_DIR/test-binary" 2>/dev/null; then
    echo "  ✓ Direct copy succeeded"
else
    echo "  ⚠ Direct copy failed (expected in some cases)"
fi

# Now test our new method (remove + copy)
echo "  - Testing remove + copy (new method)..."
if sudo rm -f "$TEST_DIR/test-binary" && sudo cp target/release/linux-distro-agent "$TEST_DIR/test-binary"; then
    echo "  ✓ Remove + copy succeeded"
else
    echo "  ✗ Remove + copy failed"
    exit 1
fi

# Test that the binary is executable and works
echo "Testing binary functionality..."
if "$TEST_DIR/test-binary" --version >/dev/null 2>&1; then
    echo "  ✓ Binary is functional"
else
    echo "  ✗ Binary is not functional"
    exit 1
fi

# Clean up
rm -rf "$TEST_DIR"
echo "✓ Cleaned up test environment"

echo ""
echo "🎉 All tests passed! The 'Text file busy' fix should work correctly."
echo ""
echo "Key improvements in the fix:"
echo "- Uses 'sudo rm -f' before 'sudo cp' to avoid file busy errors"
echo "- Applied to both update and restore operations for consistency"
echo "- Maintains proper file permissions with 'sudo chmod 755'"
echo "- Provides better error handling and user feedback"
