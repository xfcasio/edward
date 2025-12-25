#!/usr/bin/env bash
set -euo pipefail

if [ -n "${1:-}" ]; then
  REMOTE_SERVER="$1"

  ssh "$REMOTE_SERVER" 'tmux send-keys C-c'
  echo 'paused server instance..'

  echo 'starting local instance..'
  cd "$HOME/dev/edward" || exit 1

  set +e
  trap 'echo; echo "local instance stopped."' SIGINT
  cargo run
  trap - SIGINT
  set -e

  echo 'resuming server instance..'
  ssh "$REMOTE_SERVER" 'tmux send-keys "./dev" Enter'
else
  echo "no remote server address given"
  echo
  echo 'usage: ./switch-edward.sh <SSH REMOTE DESTINATION>'
fi
