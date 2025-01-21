package substrate

import (
	"encoding/json"
	"io"
	"net/http"
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
func FetchChainSpecJSON(chainSpecUrl string) (*ChainSpecRes, error) {
	// Replace "json_server" with "localhost" in the URL
	url := strings.Replace(chainSpecUrl, "json_server", "localhost", 1)

	// Use the shared HTTP client
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
