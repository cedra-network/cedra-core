#!/bin/bash
# Copyright (c) Aptos Foundation
# Copyright (c) The Move Contributors
# SPDX-License-Identifier: Apache-2.0

# A script to check whether a local commit related to Move is ready for a PR.

set -e

MOVE_PR_PROFILE=ci

BASE=$(git rev-parse --show-toplevel)

# This is currently setup for the aptos-core environment. If move is at a different
# location, this need to be changed.
MOVE_BASE=$BASE/third_party/move

echo "*************** [move-pr] Assuming move root at $MOVE_BASE"

# Run only tests which would also be run on CI
export ENV_TEST_ON_CI=1

while getopts "htcgdea" opt; do
  case $opt in
    h)
      cat <<EOF
Performs CI equivalent checks on a local client
Usage:
    check_pr <flags>
Flags:
    -h   Print this help
    -t   Run tests
    -i   In addition to -t, run integration tests (Aptos framework and e2e tests)
    -c   Run xclippy and fmt +nightly
    -g   Run the git checks script (whitespace check). This works
         only for committed clients.
    -d   Run artifact generation for move-stdlib and other Move libraries.
    -a   All of the above
    With no options script behaves like -tcg is given.
    You can use the `MOVE_PR_PROFILE` environment variable to
    determine which cargo profile to use. (The default
    is `ci`, `debug` might be faster for build.)
EOF
      exit 1
      ;;
    t)
      TEST=1
      ;;
    i)
      INTEGRATION_TEST=1
      ;;
    c)
      CHECK=1
      ;;
    d)
      GEN_ARTIFACTS=1
      ;;
    g)
      GIT_CHECKS=1
      ;;
    a)
      INTEGRATION_TEST=1
      CHECK=1
      GEN_ARTIFACTS=1
      GIT_CHECKS=1
  esac
done

if [ "$OPTIND" -eq 1 ]; then
  TEST=1
  CHECK=1
  GIT_CHECKS=1
fi

ARTIFACT_CRATE_PATHS="\
  move-stdlib\
"

MOVE_CRATES="\
  -p move-compiler-v2\
  -p move-model\
  -p move-stackless-bytecode\
  -p move-stdlib\
  -p move-bytecode-verifier\
  -p move-binary-format\
  -p move-compiler-v2\
  -p move-compiler-v2-transactional-tests\
  -p move-prover-boogie-backend\
  -p move-prover-bytecode-pipeline\
  -p move-prover\
  -p move-vm-runtime\
  -p move-vm-transactional-tests\
  -p move-vm-types\
  -p move-unit-test\
  -p move-package\
"

INTEGRATION_TEST_CRATES="\
  -p e2e-move-tests\
  -p aptos-framework\
"

if [ ! -z "$TEST" ]; then
  echo "*************** [move-pr] Running tests"
  (
    # It is important to run all tests from one cargo command to keep cargo features
    # stable.
    cd $BASE
    cargo nextest run --cargo-profile $MOVE_PR_PROFILE \
     $MOVE_CRATES
  )
fi

if [ ! -z "$INTEGRATION_TEST" ]; then
  echo "*************** [move-pr] Running extended tests"
  (
    cd $BASE
    cargo nextest run --cargo-profile $MOVE_PR_PROFILE \
     $MOVE_CRATES $INTEGRATION_TEST_CRATES
  )
fi


if [ ! -z "$CHECK" ]; then
  echo "*************** [move-pr] Running checks"
  (
    cd $BASE
    cargo xclippy
    cargo fmt +nightly
    cargo sort --grouped --workspace 
  )
fi

if [ ! -z "$GEN_ARTIFACTS" ]; then
  for dir in $ARTIFACT_CRATE_PATHS; do
    echo "*************** [move-pr] Generating artifacts for crate $dir"
    (
      cd $MOVE_BASE/$dir
      cargo run --profile $MOVE_PR_PROFILE
    )
  done
  # Add hoc treatment
  (
    cd $BASE
    cargo build --profile $MOVE_PR_PROFILE -p aptos-cached-packages
  )
fi

if [ ! -z "$GIT_CHECKS" ]; then
   echo "*************** [move-pr] Running git checks"
   $BASE/scripts/git-checks.sh
fi
