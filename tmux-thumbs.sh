#!/usr/bin/env bash

[ -f ~/.bash_profile ] && source ~/.bash_profile

PARAMS=()

function add-boolean-param {
  VALUE=$(tmux show -vg @thumbs-$1 2> /dev/null)

  if [[ "${VALUE}" == "1" ]]; then
    PARAMS+=("--$1")
  fi
}

function add-option-param {
  VALUE=$(tmux show -vg @thumbs-$1 2> /dev/null)

  if [[ ${VALUE} ]]; then
    PARAMS+=("--$1=${VALUE}")
  fi
}

function add-multi-param {
  while read -r ITEM_KEY; do
    VALUE=$(tmux show -vg $ITEM_KEY 2> /dev/null)
    PARAMS+=("--$1=${VALUE}")
  done < <(tmux show -g 2> /dev/null | grep thumbs-$1- | cut -d' ' -f1)
}

add-boolean-param "reverse"
add-boolean-param "unique"
add-option-param  "alphabet"
add-option-param  "position"
add-option-param  "fg-color"
add-option-param  "bg-color"
add-option-param  "hint-bg-color"
add-option-param  "hint-fg-color"
add-option-param  "select-fg-color"
add-option-param  "select-bg-color"
add-option-param  "command"
add-option-param  "upcase-command"
add-multi-param   "regexp"
add-boolean-param "contrast"

# Remove empty arguments from PARAMS.
# Otherwise, they would choke up tmux-thumbs when passed to it.
for i in "${!PARAMS[@]}"; do
  [ -n "${PARAMS[$i]}" ] || unset "PARAMS[$i]"
done

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

${CURRENT_DIR}/target/release/tmux-thumbs
