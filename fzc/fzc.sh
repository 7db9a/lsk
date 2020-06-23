#!/bin/bash

# $1 is fuzzy command tool (e.g. fzf or fzy).

#fuzzy_zsh_history() {
#    cat $HOME/.zsh_history | cut -c 16- | $1
#}
#
#if [ "$1" = "zsh" ]; then
#    fuzzy_zsh_history $2
#fi

cat $HOME/.zsh_history | cut -c 16- | fzf
