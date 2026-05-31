#!/usr/bin/env bash
set -e

# assumes kitty
CURR_FONT=$(sed -nr 's/font_size ([0-9]+)\.[0-9]+/\1/p' ~/.config/kitty/kitty.conf)
DEFAULT=${CURR_FONT:-14}

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

trap reset SIGINT
cargo run
reset
