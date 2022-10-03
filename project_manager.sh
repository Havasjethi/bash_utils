#!/bin/bash

# Parse arguments
source ./arg_parser.sh

project_file=${project_file:-${HAVAS_PROJECT_FILE:-'Meu'}}


# Global State
function declare_variables () {
  pattern=".*o[or].*"
  wd=~/Documents/Projects/
  message="I'm to lazy to write commits"
  no_commit=0
  folder_regex=''
  files='*'
  nickname=''
}

declare_variables


while read line; do
  if [[ ! $line ]] ; then
    if [[ ! $nickname ]]; then continue; fi;

    echo
    echo "nick: $nickname"
    echo p $pattern
    echo w $wd
    echo m $message
    echo n $no_commit
    echo "F $folder_regex"
    echo f "$files"
    echo


    # TODO :: Commit like

    # find $wd -regextype sed -maxdepth 1 -regex $pattern
    declare_variables
    continue
  fi;

  if [[ ! $line =~ ^(#|//) ]]; then

    # if [[ $line =~ ^path ]]; then
      # readarray -d ' ' -t stuff<<<"$line";
      # wd=${stuff[1]}
      # echo "Path found $wd"
    # fi


    case $line in
      \[*)
        nickname=$line;;
      path*)
        readarray -d ' ' -t stuff<<<"$line";
        wd=${stuff[1]};;
      message*)
        readarray -d ' ' -t stuff<<<"$line";
        message=${stuff[1]};;
      no_commit*)
        no_commit=1;;
      folder_regex*)
        # TODO :: Fix current method (message Hello I'm a cat) -> (Hello)
        readarray -d ' ' -t stuff<<<"$line";
        folder_regex=${stuff[1]};;
      files*)
        readarray -d ' ' -t stuff<<<"$line";
        files=${stuff[1]};;
      *)
        echo "Not recognised: $line";;
    esac

    # if [[  $line =~ ^path  ]]; then
      # readarray -d ' ' -t stuff<<<"$line"
      # project=${stuff[1]}
      # wd=$project
    # elif [[ $line =~ ]]; then
    # fi
    # TODO :: Parse line
    # continue;
  else

    # Commenct should be ignored
    continue;
  fi;

done < $project_file
# while read line;

# ; done < cat
# cat $project_file



