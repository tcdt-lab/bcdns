#!/bin/sh

# Read node key from file
NODE_KEY=$(cat /substrate/node_key.txt)

/substrate/target/release/node-template \
  --base-path /substrate \
  --dev \
  --port 30333 \
  --rpc-port 9945 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator \
  --rpc-cors all \
  --rpc-external \
  --rpc-methods Unsafe \
  --rpc-max-connections 1000 \
  --rpc-max-subscriptions-per-connection 5000 \
  --name "root-node" \
  --node-key $NODE_KEY \
  --public-addr "/ip4/0.0.0.0/tcp/30333"

