#!/bin/bash

NUM_PROG=${1:-10}
PARENT_DIR=$(realpath $(dirname $0)/..)
OUTPUT_DIR="$PARENT_DIR/output"

function get_error() {
  find $1 -name compile.log | while read f; do
    grep "error\[E" $f
  done | sort | uniq
}

rm -rf $OUTPUT_DIR
cargo run --bin generator -- -o $OUTPUT_DIR -s 1234 -p -n $NUM_PROG

for p in $OUTPUT_DIR/*; do
  if [ -d "$p" ]; then
    echo "Checking $p"
    (
        cd "$p"
        aptos move compile > compile.log 2>&1
        if [ $? -ne 0 ]; then
          echo "Compile $p: failed"
          get_error .
        else
          echo "Compile $p: success"
        fi
    )
  fi
done

echo
echo "Errors are:"
get_error $OUTPUT_DIR