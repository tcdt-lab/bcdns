package mocks

import (
	"github.com/khalidzahra/dns_client/substrate"
)

// MockSubstrateConnector is a mock implementation of the SubstrateInterface
type MockSubstrateConnector struct {
	ResolveDomainFunc    func(domain string, eval bool) (*substrate.ChainSpecRes, error)
	RegisterAssetFunc    func(domain, assetName string, nonce uint32, results chan string) uint32
	ListenForEventsFunc  func(results chan string, assetEval bool, totalRuns int)
}

// ResolveDomain mocks the ResolveDomain method
func (m *MockSubstrateConnector) ResolveDomain(domain string, eval bool) (*substrate.ChainSpecRes, error) {
	if m.ResolveDomainFunc != nil {
		return m.ResolveDomainFunc(domain, eval)
	}
	return &substrate.ChainSpecRes{
		Id:        "mock-chain-id",
		BootNodes: []string{"mock-bootnode"},
	}, nil
}

// RegisterAsset mocks the RegisterAsset method
func (m *MockSubstrateConnector) RegisterAsset(domain, assetName string, nonce uint32, results chan string) uint32 {
	if m.RegisterAssetFunc != nil {
		return m.RegisterAssetFunc(domain, assetName, nonce, results)
	}
	return nonce + 1
}

// ListenForEvents mocks the ListenForEvents method
func (m *MockSubstrateConnector) ListenForEvents(results chan string, assetEval bool, totalRuns int) {
	if m.ListenForEventsFunc != nil {
		m.ListenForEventsFunc(results, assetEval, totalRuns)
	}
}

// Ensure MockSubstrateConnector implements SubstrateInterface
var _ substrate.SubstrateInterface = (*MockSubstrateConnector)(nil)
