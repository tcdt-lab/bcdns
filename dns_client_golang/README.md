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

Listens for blockchain events and processes specific module events. Used for evaluation.

---

