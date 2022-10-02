#!/bin/bash

# Parse arguments
source ./arg_parser.sh

workdir=${workdir:-""}
message=${message:-"Too lazy to wirte commit message."}
no_commit=${no_commit:-1}
files_to_commit=${files:-"*"}

if [[ ! -n $workdir ]]; then
  echo Fatal: --workdir is missing
  exit 1
fi

if [[ $no_commit -gt 0 ]]; then
  git add "$workdir/$files"
  git commit -m "$message"

  echo Commit created
fi;

git push
exit $!
