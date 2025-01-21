# DNS Client

This package provides an implementation of a DNS client for the decentralized architecture presented in this project.

---

## Installation

1. Install dependencies:
   ```bash
   npm install
   ```

2. Configure the network:
   Update the `ROOT_DNS_NETWORK_SPEC_ADDR` in `config.js` with the address of your root DNS network.

---

## Usage

### 1. Resolving a Domain

To resolve a domain, use the following command:

```bash
npm start <domain>
```

#### Example:
```bash
npm start example.tld
```

#### Output:
- If the domain has a `targetSpec`, it will display the target specification.
- If the domain represents an asset, it will display the asset details.

---

### 2. Registering a TLD, Domain, or Asset

To register a TLD, domain, or asset, use the `npm run register` command with the appropriate flags:

#### Register a TLD:
```bash
npm run register --tld <tld> <spec> <phrase...>
```
- `<tld>`: The top-level domain to register.
- `<spec>`: The specification for the TLD.
- `<phrase...>`: Optional phrases for additional information.

##### Example:
```bash
npm run register --tld example "TLD specification" "TLD description"
```

#### Register a Domain:
```bash
npm run register --domain <domain> <spec> <phrase...>
```
- `<domain>`: The domain to register.
- `<spec>`: The specification for the domain.
- `<phrase...>`: Optional phrases for additional information.

##### Example:
```bash
npm run register --domain sub.example.tld "Domain specification" "Domain description"
```

#### Register an Asset:
```bash
npm run register --asset <domain> <assetId> <amount>
```
- `<domain>`: The domain associated with the asset.
- `<assetId>`: The identifier for the asset.
- `<amount>`: The amount of the asset.

##### Example:
```bash
npm run register --asset example.tld asset123 100
```

#### Help Command:
To display usage instructions for registration:
```bash
npm run register --help
```

