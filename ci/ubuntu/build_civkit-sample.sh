#!/bin/bash
set -o errexit

cd ../../target/debug

if cargo build --bin civkit-sample --target=x86_64-unknown-linux-gnu; then
  echo "civkit-sample build: SUCCESS"
else
  echo "civkit-sample build: FAILURE"
  exit 1
fi
