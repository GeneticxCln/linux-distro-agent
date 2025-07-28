#!/bin/bash
./target/debug/linux-distro-agent -v agent --add-task "echo testing" &
AGENT_PID=$!
sleep 1
./target/debug/linux-distro-agent agent --start &
START_PID=$!
sleep 2
kill $AGENT_PID $START_PID 2>/dev/null || true
wait

