package substrate

import (
	"encoding/json"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

type ChainSpecRes struct {
	Id        string   `json:"id"`
	BootNodes []string `json:"bootNodes"`
}

// Shared HTTP client to reuse connections
var (
	clientOnce sync.Once
	client     *http.Client
)

func getHTTPClient() *http.Client {
	clientOnce.Do(func() {
		client = &http.Client{}
	})
	return client
}

// FetchChainSpecJSON fetches the chain spec JSON from the given URL
// This is a variable to allow for mocking in tests
var FetchChainSpecJSON = fetchChainSpecJSON

// fetchChainSpecJSON fetches the chain spec JSON from either a URL or local file path
func fetchChainSpecJSON(source string) (*ChainSpecRes, error) {
	// Check if source is a URL (starts with http:// or https://)
	if strings.HasPrefix(source, "http://") || strings.HasPrefix(source, "https://") {
		// Handle URL case
		url := strings.Replace(source, "json_server", "localhost", 1)
		res, err := getHTTPClient().Get(url)
		if err != nil {
			return nil, err
		}
		defer res.Body.Close()

		body, err := io.ReadAll(res.Body)
		if err != nil {
			return nil, err
		}

		var chainSpec ChainSpecRes
		err = json.Unmarshal(body, &chainSpec)
		if err != nil {
			return nil, err
		}
		return &chainSpec, nil
	}

	// Handle file path case
	absPath, err := filepath.Abs(source)
	if err != nil {
		return nil, err
	}

	file, err := os.Open(absPath)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	body, err := io.ReadAll(file)
	if err != nil {
		return nil, err
	}

	var chainSpec ChainSpecRes
	err = json.Unmarshal(body, &chainSpec)
	if err != nil {
		return nil, err
	}

	return &chainSpec, nil
}

// TestError is a simple error type for testing
type TestError struct {
	Message string
}

// Error implements the error interface
func (e *TestError) Error() string {
	return e.Message
}
