#!/bin/bash

# Generate initial chain specification
./target/release/node-template build-spec --disable-default-bootnode --chain local > customSpec.json

# Generate network key
NODE_KEY=$(./target/release/node-template key generate-node-key)

# Get the node's public key (aura key)
AURA_KEY=$(./target/release/node-template key inspect --scheme Sr25519 "//Alice" | grep "Public" | awk '{print $2}')

# Get the node's grandpa key
GRANDPA_KEY=$(./target/release/node-template key inspect --scheme Ed25519 "//Alice" | grep "Public" | awk '{print $2}')

# Update the chain specification
./target/release/node-template build-spec --chain=customSpec.json --raw --disable-default-bootnode > customSpecRaw.json

# Save the node key to a file
echo "$NODE_KEY" > node_key.txt
