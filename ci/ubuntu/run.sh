#!/usr/bin/env bash

set -o errexit

# Call the setup script
echo "Setting up the environment..."
./setup_env.sh
echo "Environment setup completed."

# Array to store build statuses
build_statuses=()

# Build civkit-cli
echo "Building civkit-cli..."
CLI_STATUS=$(./build_civkit-cli.sh)
build_statuses+=("$CLI_STATUS")

# Build civkitd
echo "Building civkitd..."
CIVKITD_STATUS=$(./build_civkitd.sh)
build_statuses+=("$CIVKITD_STATUS")

# Build civkit-sample
echo "Building civkit-sample..."
SAMPLE_STATUS=$(./build_civkit-sample.sh)
build_statuses+=("$SAMPLE_STATUS")

# Print build statuses
for status in "${build_statuses[@]}"; do
  echo "$status"
done

# Check if any build failed
if [[ "${build_statuses[*]}" =~ .*"FAILURE".* ]]; then
  echo "One or more components failed to build."
  exit 1
fi

echo "All components built successfully."
