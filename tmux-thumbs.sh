#!/usr/bin/env bash

[ -f ~/.bash_profile ] && source ~/.bash_profile

PARAMS=()

function add-option-param {
  VALUE=$(tmux show -vg @thumbs-$1 2> /dev/null)

  if [[ ${VALUE} ]]; then
    PARAMS+=("--$1=${VALUE}")
  fi
}

add-option-param "command"
add-option-param "upcase-command"

# Remove empty arguments from PARAMS.
# Otherwise, they would choke up tmux-thumbs when passed to it.
for i in "${!PARAMS[@]}"; do
  [ -n "${PARAMS[$i]}" ] || unset "PARAMS[$i]"
done

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

${CURRENT_DIR}/target/release/tmux-thumbs --dir "${CURRENT_DIR}" "${PARAMS[@]}"

true
