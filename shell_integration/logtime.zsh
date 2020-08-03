LOGTIME_EXE=$(which logtime)
function logtime {
  conclude=/tmp/$(uuidgen)
  $LOGTIME_EXE --zsh $conclude $*
  if test -f $conclude
  then
    . $conclude
    rm $conclude
  fi
}
