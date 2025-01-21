# Enabling Blockchain Interoperability Through Discovery

This projects aims to create a discovery mechanism inspired by traditional DNS
for the interoperability of blockchain networks. The main goals of this design
are to:

1. Maximize the decentralization of the discovery mechanism
2. Ensure that the solution is scalable
3. Allow for dynamic discovery services by enabling networks to voluntarily
   opt-in or opt-out.

To achieve this, we propose a multi-layered architecture for a blockchain- based
discovery mechanism. Inspired by the traditional DNS architecture, blockchain
network domains are organized into top-level domains (TLDs) as shown in the
figure below. A root net- work manages the TLD networks by storing their
connection information while remaining agnostic of individual domain details.

![bcdns architecture diagram](./img/architecture_diagram.png)

## Launching a Node

To launch a node for testing pallet functionality, it must first be compiled by
running the following inside the node's directory:

```bash
cargo build --release
```

Then the node can be run for testing purposes using:

```bash
./target/release/node-template --dev
```

The pallets can then be interacted with through the
[polkadot explorer](https://polkadot.js.org/apps/#/explorer)

