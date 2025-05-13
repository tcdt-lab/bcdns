package mocks

import (
	"github.com/khalidzahra/dns_client/substrate"
)

// MockSubstrateConnector is a mock implementation of the SubstrateInterface
type MockSubstrateConnector struct {
	ResolveDomainFunc           func(domain string, eval bool) (*substrate.ChainSpecRes, error)
	RegisterAssetFunc           func(domain, assetName string, nonce uint32, results chan string) uint32
	ListenForEventsFunc         func(results chan string, assetEval bool, totalRuns int)
	VoteForDomainRevocationFunc func(domain string, nonce uint32, results chan string) uint32
	SendHeartbeatFunc           func(domain string, nonce uint32, results chan string) uint32
	ReportMissedHeartbeatFunc   func(domain string, nonce uint32, results chan string) uint32
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

// VoteForDomainRevocation mocks the VoteForDomainRevocation method
func (m *MockSubstrateConnector) VoteForDomainRevocation(domain string, nonce uint32, results chan string) uint32 {
	if m.VoteForDomainRevocationFunc != nil {
		return m.VoteForDomainRevocationFunc(domain, nonce, results)
	}
	return nonce + 1
}

// SendHeartbeat mocks the SendHeartbeat method
func (m *MockSubstrateConnector) SendHeartbeat(domain string, nonce uint32, results chan string) uint32 {
	if m.SendHeartbeatFunc != nil {
		return m.SendHeartbeatFunc(domain, nonce, results)
	}
	return nonce + 1
}

// ReportMissedHeartbeat mocks the ReportMissedHeartbeat method
func (m *MockSubstrateConnector) ReportMissedHeartbeat(domain string, nonce uint32, results chan string) uint32 {
	if m.ReportMissedHeartbeatFunc != nil {
		return m.ReportMissedHeartbeatFunc(domain, nonce, results)
	}
	return nonce + 1
}

// Ensure MockSubstrateConnector implements SubstrateInterface
var _ substrate.SubstrateInterface = (*MockSubstrateConnector)(nil)
