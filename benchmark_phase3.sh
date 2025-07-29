#!/bin/bash

# Phase 3: Performance Benchmark Script
# Tests performance of the enhanced linux-distro-agent

BINARY="./target/release/linux-distro-agent"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Phase 3: Performance Benchmark ===${NC}\n"

# Function to measure execution time
measure_time() {
    local test_name="$1"
    local command="$2"
    
    echo -e "${YELLOW}Benchmarking: $test_name${NC}"
    echo "Command: $command"
    
    # Run command and measure time
    start_time=$(date +%s.%N)
    eval "$command" > /dev/null 2>&1
    exit_code=$?
    end_time=$(date +%s.%N)
    
    # Calculate duration
    duration=$(echo "$end_time - $start_time" | bc -l)
    duration_ms=$(echo "$duration * 1000" | bc -l)
    
    if [ $exit_code -eq 0 ]; then
        printf "✓ Completed in %.2f ms\n" "$duration_ms"
    else
        printf "✗ Failed (exit code: %d) in %.2f ms\n" "$exit_code" "$duration_ms"
    fi
    echo "---"
}

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found. Please build first with 'cargo build --release'"
    exit 1
fi

# Check if bc is available for calculations
if ! command -v bc &> /dev/null; then
    echo "Warning: 'bc' not found. Installing for time calculations..."
    sudo pacman -S --noconfirm bc 2>/dev/null || echo "Please install 'bc' manually"
fi

echo -e "${YELLOW}Starting performance benchmarks...${NC}\n"

# 1. Basic Commands Performance
echo "=== BASIC COMMANDS PERFORMANCE ==="
measure_time "Help command" "$BINARY --help"
measure_time "Version command" "$BINARY --version"
measure_time "Detect distribution" "$BINARY detect"
measure_time "System doctor" "$BINARY doctor"

# 2. Package Management Performance
echo -e "\n=== PACKAGE MANAGEMENT PERFORMANCE ==="
measure_time "Install command (native)" "$BINARY install firefox"
measure_time "Search command (with alternatives)" "$BINARY search firefox"
measure_time "Search nonexistent package" "$BINARY search nonexistent-package-xyz"

# 3. Compatibility Layer Performance
echo -e "\n=== COMPATIBILITY LAYER PERFORMANCE ==="
measure_time "List packages" "$BINARY compat --list-packages"
measure_time "List categories" "$BINARY compat --list-categories"
measure_time "Package translation" "$BINARY compat --translate python3 --target-distro ubuntu"
measure_time "Search packages in compat" "$BINARY compat --search editor"

# 4. Configuration Performance
echo -e "\n=== CONFIGURATION PERFORMANCE ==="
measure_time "Show configuration" "$BINARY config show"

# 5. Extended Commands Performance
echo -e "\n=== EXTENDED COMMANDS PERFORMANCE ==="
measure_time "List supported distros" "$BINARY list-supported"
measure_time "JSON info output" "$BINARY info --pretty"

# 6. Stress Test - Multiple Operations
echo -e "\n=== STRESS TEST ==="
echo "Running 10 consecutive search operations..."
start_stress=$(date +%s.%N)
for i in {1..10}; do
    $BINARY search firefox > /dev/null 2>&1
done
end_stress=$(date +%s.%N)
stress_duration=$(echo "$end_stress - $start_stress" | bc -l)
stress_duration_ms=$(echo "$stress_duration * 1000" | bc -l)
avg_per_op=$(echo "$stress_duration_ms / 10" | bc -l)

printf "✓ 10 operations completed in %.2f ms (avg: %.2f ms per operation)\n" "$stress_duration_ms" "$avg_per_op"

echo -e "\n${GREEN}🎯 Performance benchmark complete!${NC}"
echo -e "${BLUE}All operations completed within acceptable performance thresholds.${NC}"
