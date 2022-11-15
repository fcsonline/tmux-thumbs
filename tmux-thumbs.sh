#!/usr/bin/env bash
set -Eeu -o pipefail

# Setup env variables to be compatible with compiled and bundled installations
CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
RELEASE_DIR="${CURRENT_DIR}/target/release"

THUMBS_BINARY="${RELEASE_DIR}/thumbs"
TMUX_THUMBS_BINARY="${RELEASE_DIR}/tmux-thumbs"
VERSION=$(grep 'version =' "${CURRENT_DIR}/Cargo.toml" | grep -o "\".*\"" | sed 's/"//g')

if [ ! -f "$THUMBS_BINARY" ]; then
  tmux split-window "cd ${CURRENT_DIR} && bash ./tmux-thumbs-install.sh"
  exit
elif [[ $(${THUMBS_BINARY} --version) != "thumbs ${VERSION}"  ]]; then
  tmux split-window "cd ${CURRENT_DIR} && bash ./tmux-thumbs-install.sh update"
  exit
fi

function get-opt-value() {
  tmux show -v "${2:-g}" "${1}" 2> /dev/null
}

function get-thumb-opt-value() {
  get-opt-value "@thumbs-${1}"
}

function set-opt-value() {
  tmux set "${3:-g}" "${1}" "${2}" 2> /dev/null
}

function get-opt-arg() {
  local opt type value
  opt="${1}"; type="${2}"
  value="$(get-thumb-opt-value "${opt}")" || true

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

# Temporarily suppress tmux visual effects to work around display lag.
function suppress-visual-effects() {
  opt_ma="monitor-activity"
  opt_va="visual-activity"
  opt_vb="visual-bell"
  opt_vs="visual-silence"
  opt_ma_val=$(get-opt-value "${opt_ma}" -p)
  opt_va_val=$(get-opt-value "${opt_va}" -p)
  opt_vb_val=$(get-opt-value "${opt_vb}" -p)
  opt_vs_val=$(get-opt-value "${opt_vs}" -p)

  function cleanup {
    set-opt-value "${opt_ma}" "${opt_ma_val}" -p
    set-opt-value "${opt_va}" "${opt_va_val}" -p
    set-opt-value "${opt_vb}" "${opt_vb_val}" -p
    set-opt-value "${opt_vs}" "${opt_vs_val}" -p
  }
  trap cleanup EXIT

  # https://github.com/fcsonline/tmux-thumbs/issues/88#issuecomment-871516639
  set-opt-value "${opt_ma}" off -p
  set-opt-value "${opt_va}" off -p
  set-opt-value "${opt_vb}" off -p
  set-opt-value "${opt_vs}" on -p
}

suppress-visual-effects

add-param command        string
add-param upcase-command string
add-param multi-command  string
add-param osc52          boolean

"${TMUX_THUMBS_BINARY}" "${PARAMS[@]}" || true
