#!/bin/bash

function havas_parser() {
  while [ $# -gt 0 ]; do
     if [[ $1 == *"--"* ]]; then
      # Parameter name without the '--' part
      param="${1/--/}"

      # Is second param exists && Value doesn't have '--' Parsing the 'value'
      if [[ $2 ]] && [[ $2 != *"--"* ]]; then
        declare -g $param="$2"
      else
        declare -g $param=1
      fi;
     fi
    shift
  done
}


havas_parser $@
