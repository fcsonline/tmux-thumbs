#!/usr/bin/env bash

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

DEFAULT_THUMBS_KEY="space"
THUMBS_KEY=$(tmux show-option -gqv @thumbs-key)
THUMBS_KEY=${THUMBS_KEY:-$DEFAULT_THUMBS_KEY}

tmux bind-key $THUMBS_KEY run-shell -b "${CURRENT_DIR}/tmux-thumbs.sh"

BINARY="${CURRENT_DIR}/target/release/thumbs"

if [ ! -f "$BINARY" ]; then
  tmux split-window "cd ${CURRENT_DIR} && cargo build --release && echo 'Press any key to continue...' && read -k1"
fi
