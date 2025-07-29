#!/bin/bash

# Phase 3: Comprehensive Testing Script
# Tests all functionality of the enhanced linux-distro-agent

# set -e  # Don't exit on error for testing

BINARY="./target/release/linux-distro-agent"
PASSED=0
FAILED=0
TOTAL=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test function
run_test() {
    local test_name="$1"
    local command="$2"
    local expected_exit_code="${3:-0}"
    
    echo -e "${YELLOW}Testing: $test_name${NC}"
    echo "Command: $command"
    
    TOTAL=$((TOTAL + 1))
    
    eval "$command" > /dev/null 2>&1
    actual_exit_code=$?
    
    if [ $actual_exit_code -eq $expected_exit_code ]; then
        echo -e "${GREEN}✓ PASSED${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗ FAILED (expected exit code $expected_exit_code, got $actual_exit_code)${NC}"
        FAILED=$((FAILED + 1))
    fi
    echo "---"
}

# Test command existence
echo "=== Phase 3: Comprehensive Testing ==="
echo "Binary: $BINARY"

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found. Please build first with 'cargo build --release'"
    exit 1
fi

echo -e "${YELLOW}Starting comprehensive test suite...${NC}\n"

# 1. Basic Commands
echo "=== BASIC COMMANDS ==="
run_test "Help command" "$BINARY --help"
run_test "Version command" "$BINARY --version"
run_test "Detect distribution" "$BINARY detect"
run_test "System doctor" "$BINARY doctor"

# 2. Package Management
echo -e "\n=== PACKAGE MANAGEMENT ==="
run_test "Install command (native)" "$BINARY install firefox"
run_test "Search command" "$BINARY search firefox"
run_test "Search with alternatives" "$BINARY search telegram"
run_test "Install nonexistent package" "$BINARY install nonexistent-package-xyz"
run_test "Search nonexistent package" "$BINARY search nonexistent-package-xyz"

# 3. Compatibility Layer
echo -e "\n=== COMPATIBILITY LAYER ==="
run_test "List packages" "$BINARY compat --list-packages"
run_test "List categories" "$BINARY compat --list-categories"
run_test "Package translation (Ubuntu)" "$BINARY compat --translate python3 --target-distro ubuntu"
run_test "Package translation (Fedora)" "$BINARY compat --translate vscode --target-distro fedora"
run_test "Search packages in compat" "$BINARY compat --search editor"

# 4. Configuration
echo -e "\n=== CONFIGURATION ==="
run_test "Show configuration" "$BINARY config show"

# 5. Extended Commands
echo -e "\n=== EXTENDED COMMANDS ==="
run_test "List supported distros" "$BINARY list-supported"
run_test "JSON info output" "$BINARY info --pretty"

# 6. Error Handling
echo -e "\n=== ERROR HANDLING ==="
run_test "Invalid command" "$BINARY invalid-command" 2
run_test "Missing arguments" "$BINARY search" 2

# 7. Help System
echo -e "\n=== HELP SYSTEM ==="
run_test "Search help" "$BINARY search --help"
run_test "Install help" "$BINARY install --help"
run_test "Compat help" "$BINARY compat --help"

# Summary
echo -e "\n=== TEST SUMMARY ==="
echo -e "Total tests: $TOTAL"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed! Phase 3 validation complete.${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Please review and fix issues.${NC}"
    exit 1
fi
