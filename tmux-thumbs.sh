#!/usr/bin/env bash

CURRENT_PANE_ID=$(tmux list-panes -F "#{pane_id}:#{?pane_active,active,nope}" | grep active | cut -d: -f1)
COMMAND="tmux-thumbs -a qwerty -r -u --tmux-pane ${CURRENT_PANE_ID}"
NEW_ID=$(tmux new-window -P -d -n "[thumbs]" ${COMMAND})
NEW_PANE_ID=$(tmux list-panes -a | grep ${NEW_ID} | grep --color=never -o '%[0-9]\+')

tmux swap-pane -d -s ${CURRENT_PANE_ID} -t ${NEW_PANE_ID}
