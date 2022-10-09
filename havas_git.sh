#!/bin/bash

echo $@
echo $#

# Parse arguments
# source ./arg_parser.sh
source ~/bin/havas_libs/arg_parser.sh

workdir=${workdir:-""}
message=${message:-"Too lazy to wirte commit message."}
no_commit=${no_commit:-0}
files_to_commit=${files:-"*"}
absolute_index=${absolute_index:-0}
stage_files=${stage_files:-0}

if [[ ! -n $workdir ]]; then
  echo Fatal: --workdir is missing
  exit 1
fi

cd
cd $workdir



if [[ $no_commit -lt 1 ]]; then
  if [[ $stage_files -gt 0 ]]; then
    if [[ $absolute_index -gt 0 ]]; then
      echo "Adding: $files"
      git add "$files"

    else
      git add "$workdir/$files"
    fi
  fi

  git commit -m "$message"

  echo Commit created
fi;



git push
if [[ $! -eq 5 ]]; then
  git push --set-upstream origin $(git rev-parse --abbrev-ref HEAD)
fi
echo $!
exit $!
