#!/usr/bin/env bash

set -o errexit

# Check if the civkit-cli binary exists
if [ -f "civkit-cli" ]; then
    echo "civkit-cli build: SUCCESS"
else
    echo "civkit-cli build: FAILURE"
    exit 1
fi
