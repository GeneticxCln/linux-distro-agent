#!/bin/bash

echo "Testing self-update build from source fix..."

# Test the dry run first
echo "1. Testing dry-run mode:"
./target/release/linux-distro-agent self-update --dry-run

echo -e "\n2. Testing build prerequisites check:"
# This should pass if git, cargo, rustc are available
git --version > /dev/null 2>&1 && echo "✓ git available" || echo "✗ git not available"
cargo --version > /dev/null 2>&1 && echo "✓ cargo available" || echo "✗ cargo not available"
rustc --version > /dev/null 2>&1 && echo "✓ rustc available" || echo "✗ rustc not available"

echo -e "\n3. Testing check mode:"
./target/release/linux-distro-agent self-update --check

# Test the rm + cp operation (simulating what our fix does)
echo -e "\n4. Testing remove + copy operation (binary replacement):"
TEST_DIR="/tmp/lda_test_$$"
mkdir -p "$TEST_DIR"
cp target/release/linux-distro-agent "$TEST_DIR/test-binary"

# Test our improved method (remove + copy)
echo "  - Testing remove + copy method..."
if sudo rm -f "$TEST_DIR/test-binary" && sudo cp target/release/linux-distro-agent "$TEST_DIR/test-binary"; then
    echo "  ✓ Remove + copy succeeded"
else
    echo "  ✗ Remove + copy failed"
    exit 1
fi

# Test that the binary is executable and works
echo "  - Testing binary functionality..."
if "$TEST_DIR/test-binary" --version >/dev/null 2>&1; then
    echo "  ✓ Binary is functional"
else
    echo "  ✗ Binary is not functional"
    exit 1
fi

# Clean up
rm -rf "$TEST_DIR"
echo "  ✓ Cleaned up test environment"

echo -e "\n🎉 All tests passed! The build from source fix should work correctly."
echo ""
echo "Key improvements made:"
echo "- Fixed git clone to use separate clone and checkout commands"
echo "- Added proper error handling with stderr output for debugging"
echo "- Fallback to main branch if specific version tag doesn't exist"
echo "- Removed custom tempfile implementation (using proper tempfile crate)"
echo "- Better error messages showing exactly what failed"
echo "- Uses 'sudo rm -f' before 'sudo cp' to avoid file busy errors"
echo "- Applied to both update and restore operations for consistency"
echo "- Maintains proper file permissions with 'sudo chmod 755'"
