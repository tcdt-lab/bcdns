package substrate

import (
	"testing"
)

// TestSubstrateConnectorResolveDomain tests the ResolveDomain method of SubstrateConnector
func TestSubstrateConnectorResolveDomain(t *testing.T) {
	// Create a new connector with caching enabled
	connector := NewSubstrateConnector(true)

	// Override the FetchChainSpecJSON function for testing
	originalFetchFunc := FetchChainSpecJSON
	defer func() { FetchChainSpecJSON = originalFetchFunc }()

	// Mock the FetchChainSpecJSON function to return a predefined root chain spec
	FetchChainSpecJSON = func(chainSpecUrl string) (*ChainSpecRes, error) {
		return &ChainSpecRes{
			Id:        "test-root-chain",
			BootNodes: []string{"/ip4/127.0.0.1/tcp/9944/ws"},
		}, nil
	}

	// Mock the getTldFromRootFunc to return a predefined TLD response
	connector.getTldFromRootFunc = func(rootSpec ChainSpecRes, keyParam string) (*TldRes, error) {
		return &TldRes{
			ChainSpec: `{"id":"test-tld-chain","bootNodes":["/ip4/127.0.0.1/tcp/9945/ws"]}`,
		}, nil
	}

	// Mock the getTargetFromTldFunc to return a predefined domain response
	connector.getTargetFromTldFunc = func(tldSpec ChainSpecRes, keyParam string) (*DomainRes, error) {
		return &DomainRes{
			ChainSpec: `{"id":"test-domain-chain","bootNodes":["/ip4/127.0.0.1/tcp/9946/ws"]}`,
			Available: true,
		}, nil
	}

	// Test cases
	testCases := []struct {
		name            string
		domain          string
		expectedChainID string
	}{
		{
			name:            "Simple domain",
			domain:          "example.com",
			expectedChainID: "test-domain-chain",
		},
		{
			name:            "Subdomain",
			domain:          "sub.example.com",
			expectedChainID: "test-domain-chain",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			// Test successful domain resolution
			result, err := connector.ResolveDomain(tc.domain, false)
			if err != nil {
				t.Fatalf("Expected no error, got %v", err)
			}

			if result.Id != tc.expectedChainID {
				t.Errorf("Expected domain chain ID '%s', got '%s'", tc.expectedChainID, result.Id)
			}

			// Verify bootNodes are correctly parsed
			if len(result.BootNodes) == 0 {
				t.Errorf("Expected bootNodes to be populated, got empty slice")
			}
		})
	}

	// Test error handling for invalid domain
	t.Run("Invalid domain", func(t *testing.T) {
		_, err := connector.ResolveDomain("invalid", false)
		if err == nil {
			t.Errorf("Expected error for invalid domain, got nil")
		}
	})
}
