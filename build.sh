#!/bin/bash

# Determine workspace optimizer for the CPU Architecture
if [[ $(uname -m) == "arm64" ]]; then
  ARCHITECTURE_EXTENSION="-arm64"
else
  ARCHITECTURE_EXTENSION=""
fi

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer$ARCHITECTURE_EXTENSION:0.14.0