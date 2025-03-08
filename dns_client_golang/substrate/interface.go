package substrate

// SubstrateInterface defines the interface for the DNS client
type SubstrateInterface interface {
	// ResolveDomain resolves a domain and fetches its associated chain specification
	ResolveDomain(domain string, eval bool) (*ChainSpecRes, error)

	// RegisterAsset registers a new asset for a domain and tracks the block number of registration
	RegisterAsset(domain, assetName string, nonce uint32, results chan string) uint32

	// ListenForEvents listens for blockchain events and processes specific module events
	ListenForEvents(results chan string, assetEval bool, totalRuns int)
}

// Ensure SubstrateConnector implements SubstrateInterface
var _ SubstrateInterface = (*SubstrateConnector)(nil)
