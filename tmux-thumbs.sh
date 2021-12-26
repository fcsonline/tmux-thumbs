#!/usr/bin/env bash
set -Eeu -o pipefail

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

BINARY="${CURRENT_DIR}/target/release/thumbs"

if [ ! -f "$BINARY" ]; then
  tmux split-window "cd ${CURRENT_DIR} && bash ./tmux-thumbs-install.sh"
  exit
fi

function get-opt-value() {
  tmux show -vg "@thumbs-${1}" 2> /dev/null
}

function get-opt-arg() {
  local opt type value
  opt="${1}"; type="${2}"
  value="$(get-opt-value "${opt}")" || true

  if [ "${type}" = string ]; then
    [ -n "${value}" ] && echo "--${opt}=${value}"
  elif [ "${type}" = boolean ]; then
    [ "${value}" = 1 ] && echo "--${opt}"
  else
    return 1
  fi
}

PARAMS=(--dir "${CURRENT_DIR}")

function add-param() {
  local type opt arg
  opt="${1}"; type="${2}"
  if arg="$(get-opt-arg "${opt}" "${type}")"; then
    PARAMS+=("${arg}")
  fi
}

add-param command        string
add-param upcase-command string
add-param multi-command  string
add-param osc52          boolean

"${CURRENT_DIR}/target/release/tmux-thumbs" "${PARAMS[@]}" || true
