#!/usr/bin/env bash

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

tmux bind-key space run-shell "${CURRENT_DIR}/tmux-thumbs.sh"
