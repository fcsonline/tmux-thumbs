#!/usr/bin/env bash

source ~/.bash_profile

COMMAND="tmux-thumbs -a qwerty -r -u"

CURRENT_PANE_ID=$(tmux list-panes -F "#{pane_id}:#{?pane_active,active,nope}" | grep active | cut -d: -f1)
NEW_ID=$(tmux new-window -P -d -n "[thumbs]" ${COMMAND} --tmux-pane ${CURRENT_PANE_ID})
NEW_PANE_ID=$(tmux list-panes -a | grep ${NEW_ID} | grep --color=never -o '%[0-9]\+')

tmux swap-pane -d -s ${CURRENT_PANE_ID} -t ${NEW_PANE_ID}
