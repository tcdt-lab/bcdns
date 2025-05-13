# DNS Client - Golang

This package provides a Golang implementation of a DNS client for the decentralized architecture presented in this project.

---

## Configuration

### Environment Variables

The following environment variable must be set to specify the root network of the architecture for 
`SubstrateConnector`:

- `ROOT_SPEC_URL`: URL of the root chain specification.
---

## Usage

### Installation

**Install Dependencies**:
   Use `go mod` to install necessary modules.
   ```bash
   go mod tidy
   ```

### Fetching Chain Specifications

Fetch chain specifications for a domain:

```bash
go run main.go --domain <domain>
```

#### Example:
```bash
go run main.go --domain example.com
```

---

## Quick Start

Here is a basic example to get started:

```go
package main

import (
	"fmt"
	"log"

	"github.com/khalidzahra/dns_client/substrate"
)

func main() {
	// Initialize the SubstrateConnector
	connector := substrate.NewSubstrateConnector(true)

	// Resolve a domain
	domain := "example.tld"
	chainSpec, err := connector.ResolveDomain(domain, true)
	if err != nil {
		log.Fatalf("Error resolving domain: %v", err)
	}

	fmt.Printf("Resolved Chain Spec: %+v\n", chainSpec)
}
```

---

## API Overview

### Initialization

```go
func NewSubstrateConnector(useCache bool) *SubstrateConnector
```

Creates a new instance of `SubstrateConnector` with optional caching.

### Domain Resolution

```go
func (c *SubstrateConnector) ResolveDomain(domain string, eval bool) (*ChainSpecRes, error)
```

Resolves a domain to fetch its chain specification.

### TLD Resolution

```go
func (c *SubstrateConnector) resolveTldSpec(tld string, eval bool) (*ChainSpecRes, error)
```

Fetches chain specifications for a top-level domain (TLD).

### Event Listening

```go
func (c *SubstrateConnector) ListenForEvents(results chan string, assetEval bool, totalRuns int)
```

Listens for blockchain events and processes specific module events.

## Decentralized Incentive Mechanism Evaluation

The DNS client now supports evaluating the decentralized incentive mechanism implemented in the TLD and assetdiscovery pallets. The following features are available:

### Domain Revocation Voting

Vote for domain revocation to evaluate the decentralized governance mechanism:

```bash
go run main.go --voteRevoke --domain example.com --runs 10 --rps 1 --outFile revocation_votes.csv
```

This will submit 10 votes for revoking the specified domain as an asset provider, at a rate of 1 vote per second, and save the results to `revocation_votes.csv`.

### Heartbeat Monitoring

Send heartbeats for a domain to prove the maintainer is online:

```bash
go run main.go --heartbeat --domain example.com --runs 10 --rps 1 --outFile heartbeats.csv
```

This will send 10 heartbeats for the specified domain, at a rate of 1 heartbeat per second, and save the results to `heartbeats.csv`.

### Missed Heartbeat Reporting

Report missed heartbeats for a domain:

```bash
go run main.go --reportMissed --domain example.com --runs 10 --rps 1 --outFile missed_heartbeats.csv
```

This will submit 10 reports for missed heartbeats for the specified domain, at a rate of 1 report per second, and save the results to `missed_heartbeats.csv`.

### Asset Registration

Register assets for a domain:

```bash
go run main.go --assetEval --domain example.com --runs 10 --rps 1 --outFile asset_registration.csv
```

This will register 10 assets for the specified domain, at a rate of 1 asset per second, and save the results to `asset_registration.csv`.

### Event Monitoring

Listen for events emitted by the chain:

```bash
go run main.go --listen --runs 100 --outFile events.csv
```

This will listen for up to 100 events emitted by the chain and save the results to `events.csv`. The events include:
- Domain validation requests
- Asset registrations
- Asset provider revocations
- Revocation votes
- Domain heartbeats
- Expired heartbeat notifications

## API Overview for Incentive Mechanism

### Domain Revocation Voting

```go
func (c *SubstrateConnector) VoteForDomainRevocation(domain string, nonce uint32, results chan string) uint32
```

Submits a vote to revoke a domain as an asset provider.

### Heartbeat Monitoring

```go
func (c *SubstrateConnector) SendHeartbeat(domain string, nonce uint32, results chan string) uint32
```

Sends a heartbeat for a domain to prove the maintainer is online.

### Missed Heartbeat Reporting

```go
func (c *SubstrateConnector) ReportMissedHeartbeat(domain string, nonce uint32, results chan string) uint32
```

Reports a domain with missed heartbeat.

## Evaluation Results

The evaluation results are saved in CSV format with the following columns:
- `Run`: The run index or identifier
- `Execution Time (ms)`: The execution time in milliseconds

For event monitoring, the results include:
- Event type (asset registration, revocation vote, heartbeat, etc.)
- Domain or asset identifier
- Block number or timestamp

These metrics can be used to evaluate the performance and effectiveness of the decentralized incentive mechanism in the BCDNS system.
