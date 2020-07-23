function logtime
  set conclude /tmp/(uuidgen)
  env logtime --fish $conclude $argv
  if test -f $conclude
    . $conclude
    rm $conclude
  end
end
