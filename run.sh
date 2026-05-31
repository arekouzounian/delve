#!/usr/bin/env bash

DEFAULT=14
FONT_SIZE=12
PROFILE=0

for arg in "$@"; do
  case "$arg" in
  --profile)
    PROFILE=1
    ;;
  *)
    FONT_SIZE="$arg"
    ;;
  esac
done

kitten @ set-font-size "$FONT_SIZE"

reset() {
  kitten @ set-font-size "$DEFAULT"
}

if [ "$PROFILE" -eq 1 ]; then
  export RUSTFLAGS="$RUSTFLAGS --cfg profiling_enabled"
fi

trap reset SIGINT SIGTERM EXIT

cargo run
