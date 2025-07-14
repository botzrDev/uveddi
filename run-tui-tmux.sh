#!/bin/bash
# Script to run TUI in a tmux session for remote access

set -e

SESSION_NAME="uveddi-tui"

echo "Starting TUI in tmux session '$SESSION_NAME'..."

# Kill existing session if it exists
tmux kill-session -t "$SESSION_NAME" 2>/dev/null || true

# Create new session and run TUI
tmux new-session -d -s "$SESSION_NAME" -c "$(pwd)"
tmux send-keys -t "$SESSION_NAME" "cargo run --bin tui_test --features tui" Enter

echo "TUI started in tmux session '$SESSION_NAME'"
echo "To attach: tmux attach-session -t $SESSION_NAME"
echo "To detach: Ctrl+b then d"
echo "To kill session: tmux kill-session -t $SESSION_NAME"

# Optionally auto-attach
read -p "Attach to session now? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    tmux attach-session -t "$SESSION_NAME"
fi