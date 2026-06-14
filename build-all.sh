#!/usr/bin/env bash

set -e

CRATES_DIR="./crates"

for crate in $(ls $CRATES_DIR); do
  echo "building $crate..."
  pushd "$CRATES_DIR/$crate"
  cargo build
  popd
done

echo "all builds successful."
