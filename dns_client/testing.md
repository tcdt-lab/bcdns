# Testing Guide

The project includes unit tests for various components. The tests are designed to verify the functionality of the DNS client without requiring a connection to the actual blockchain network.

## Running Tests

To run all tests in the project:

```bash
cd dns_client
npm test
```

### Running Tests for a Specific Component

To run tests for a specific component, you can use the following command:

```bash
npm test <component_name>
```

## Manual Testing

To manually test the DNS client, you can use the following command:

```bash
git clone https://github.com/khalidzahra/dns_client.git
cd dns_client
git checkout <branch_name>
```

### Example

```bash
git clone https://github.com/khalidzahra/dns_client.git
cd dns_client
git checkout main
git pull
npm install
npm run build
npm test
```

### Output

```bash
Resolved Chain Spec: %+v
```

However, for this to work, you need to have a valid root network specification URL set in the environment variable `ROOT_SPEC_URL`. Moreover, you need to have a valid TLD network deployed and registered on the root network. This can be done by deploying two nodes, one for the root network and one for the TLD network, and registering the TLD network on the root network.

### Node Deployment

To deploy a node, you can use the following command:

```bash
cd ../polkadot-sdk-solochain-template
cargo build --release
./target/release/node-template --dev
```

### Deploying the Architecture

> [!IMPORTANT]  
> The node must be built before running anything in this step!

To deploy the architecture, run the `init.go` script as follows:

```bash
cd ../polkadot-sdk-solochain-template/scripts && go run init.go --init
```

The architecture will be launched and can then be interacted with through the dns client applications (try resolving example.com).

Once done, the architecture can be cleaned up using:

```bash 
go run init.go --cleanup
```

### TLD Registration

To register a TLD, you can use the `register_tld` extrinsic provided by the rootdns pallet. The extrinsic can easily be called using the [polkadot explorer](https://polkadot.js.org/apps/#/explorer).

### Domain Registration

To register a domain, you can use the `register_domain` extrinsic provided by the tld pallet. The extrinsic can easily be called using the [polkadot explorer](https://polkadot.js.org/apps/#/explorer).
