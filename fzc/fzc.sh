#!/bin/bash

# $1 is the number of lines to display.
fuzzy_zsh_history() {
    cat $HOME/.zsh_history | cut -c 16- | fzf
}

if [ "$1" = "zsh" ]; then
    fuzzy_zsh_history
fi
