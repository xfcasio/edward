#!/usr/bin/env bash
set -euo pipefail

if [ -n "${1:-}" ]; then
  REMOTE_SERVER="$1"

  mkdir -p ~/pub
  cp ~/.target/release/rhbot ~/pub/dev
  sed -i 's/toji/sgma/g' ~/pub/dev

  ssh "$REMOTE_SERVER" '
  set -euo pipefail
  printf "PREVIOUS DEPLOYMENT HASH :\t"
  [ -f ~/dev ] && sha256sum ~/dev || echo "<none>"
  cat > ~/dev.1
  chmod +x ~/dev.1
  tmux send-keys C-c
  sleep 0.5
  mv ~/dev.1 ~/dev
  tmux send-keys "./dev" Enter
  printf "NEW DEPLOYMENT HASH      :\t"
  sha256sum ~/dev
  echo "Successfully deployed"
  ' < ~/pub/dev

  rm -f ~/pub/dev
  rmdir ~/pub 2>/dev/null || true
else
  echo "no remote server address given"
  echo
  echo 'usage: ./deploy-edward.sh <SSH REMOTE DESTINATION>'
fi
