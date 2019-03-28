#!/usr/bin/env bash

source ~/.bash_profile

function boolean {
  VALUE=$(tmux show -vg @thumbs-$1 2> /dev/null)

  if [[ "${VALUE}" == "1" ]]; then
    echo "--$1"
  fi
}

function option {
  VALUE=$(tmux show -vg @thumbs-$1 2> /dev/null)

  if [[ ${VALUE} ]]; then
    echo "--$1 ${VALUE}"
  fi
}

function multi {
  VALUES=""

  while read -r ITEM_KEY; do
    VALUE=$(tmux show -vg $ITEM_KEY 2> /dev/null)
    VALUES="${VALUES} --$1 ${VALUE}"
  done < <(tmux show -g 2> /dev/null | grep thumbs-$1- | cut -d' ' -f1)

  echo ${VALUES}
}

PARAMS=()
PARAMS[0]=$(boolean reverse)
PARAMS[1]=$(boolean unique)
PARAMS[2]=$(option alphabet)
PARAMS[3]=$(option position)
PARAMS[4]=$(option fg-color)
PARAMS[5]=$(option bg-color)
PARAMS[6]=$(option hint-bg-color)
PARAMS[7]=$(option hint-fg-color)
PARAMS[8]=$(option select-fg-color)
PARAMS[9]=$(option command)
PARAMS[10]=$(option upcase-command)
PARAMS[11]=$(multi regexp)

CURRENT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
TARGET_RELEASE="/target/release/"
CURRENT_PANE_ID=$(tmux list-panes -F "#{pane_id}:#{?pane_active,active,nope}" | grep active | cut -d: -f1)
COMMAND="tmux-thumbs ${PARAMS[*]} --tmux-pane ${CURRENT_PANE_ID}"
NEW_ID=$(tmux new-window -P -d -n "[thumbs]" ${CURRENT_DIR}${TARGET_RELEASE}${COMMAND})
NEW_PANE_ID=$(tmux list-panes -a | grep ${NEW_ID} | grep --color=never -o '%[0-9]\+')

tmux swap-pane -d -s ${CURRENT_PANE_ID} -t ${NEW_PANE_ID}
