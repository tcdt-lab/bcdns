package substrate

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

// TestFetchChainSpecJSON tests the FetchChainSpecJSON function
func TestFetchChainSpecJSON(t *testing.T) {
	// Create a test server that returns a mock chain spec
	testServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		mockSpec := ChainSpecRes{
			Id:        "test-chain",
			BootNodes: []string{"bootnode1", "bootnode2"},
		}
		json.NewEncoder(w).Encode(mockSpec)
	}))
	defer testServer.Close()

	// Test the function with the mock server URL
	spec, err := FetchChainSpecJSON(testServer.URL)
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}

	if spec.Id != "test-chain" {
		t.Errorf("Expected ID 'test-chain', got '%s'", spec.Id)
	}

	if len(spec.BootNodes) != 2 {
		t.Errorf("Expected 2 bootnodes, got %d", len(spec.BootNodes))
	}
}

// TestParseDomain tests the parseDomain function
func TestParseDomain(t *testing.T) {
	testCases := []struct {
		domain      string
		expectName  string
		expectTLD   string
		expectError bool
	}{
		{"example.com", "example", "com", false},
		{"sub.example.com", "sub.example", "com", false},
		{"invalid", "", "", true},
		{"", "", "", true},
	}

	for _, tc := range testCases {
		name, tld, err := parseDomain(tc.domain)
		
		if tc.expectError && err == nil {
			t.Errorf("Expected error for domain '%s', got none", tc.domain)
		}
		
		if !tc.expectError && err != nil {
			t.Errorf("Expected no error for domain '%s', got %v", tc.domain, err)
		}
		
		if name != tc.expectName {
			t.Errorf("Expected name '%s', got '%s'", tc.expectName, name)
		}
		
		if tld != tc.expectTLD {
			t.Errorf("Expected TLD '%s', got '%s'", tc.expectTLD, tld)
		}
	}
}

// TestGetConnectionAddress tests the getConnectionAddress function
func TestGetConnectionAddress(t *testing.T) {
	testCases := []struct {
		bootNodeAddr string
		expected     string
	}{
		{"/ip4/127.0.0.1/tcp/9944/ws", "ws://127.0.0.1:9944"},
		{"/ip4/192.168.1.1/tcp/8080/ws", "ws://192.168.1.1:8080"},
		{"invalid", "ws://localhost:9944"}, // Default fallback
	}

	for _, tc := range testCases {
		result := getConnectionAddress(tc.bootNodeAddr)
		if result != tc.expected {
			t.Errorf("For bootnode '%s', expected '%s', got '%s'", tc.bootNodeAddr, tc.expected, result)
		}
	}
}
