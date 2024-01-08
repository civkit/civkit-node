#!/bin/bash

# might be outdated as we generate a default config.toml on first start

# Define the path to the config file
CONFIG_FILE="./config.toml"

# TODO does not match Config::default()
# Check if the file exists
if [[ ! -f $CONFIG_FILE ]]; then
    echo "config.toml not found. Creating a default one..."

    # Define the default content for the config file
    config_content='[performance]
max_db_size = 10000
max_event_age = 3600

[spam_protection]
requestcredentials = true

[connections]
maxclientconnections = 100

[civkitd]
network = "testnet"
noise_port = 9735
nostr_port = 50021
cli_port = 50031

[logging]
level = "info"'

    # Write the default content to the config file
    echo "$config_content" > $CONFIG_FILE
else
    echo "config.toml already exists."
fi
