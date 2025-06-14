package eval

import (
	"encoding/csv"
	"fmt"
	"os"
	"sync"
	"time"

	"github.com/khalidzahra/dns_client/substrate"
)

type EvalResult struct {
	Idx  int
	Time int64
}

type AssetEvalResult struct {
	RequestId int
	Time      int
}

func WriteToCSV(csvFilename string, resultArr []*EvalResult) {
	file, err := os.Create(csvFilename)
	if err != nil {
		fmt.Println("Error creating CSV file:", err)
		return
	}
	defer file.Close()

	writer := csv.NewWriter(file)
	defer writer.Flush()

	if err := writer.Write([]string{"Run", "Execution Time (ms)"}); err != nil {
		fmt.Println("Error writing CSV header:", err)
		return
	}

	for _, result := range resultArr {
		if err := writer.Write([]string{fmt.Sprintf("%d", result.Idx), fmt.Sprintf("%d", result.Time)}); err != nil {
			fmt.Println("Error writing CSV row:", err)
			return
		}
	}

	fmt.Printf("Execution times saved to %s\n", csvFilename)
}

func RunFuncPerSecond(fn func(int, *sync.WaitGroup), runs, runsPerSecond int) {
	var wg sync.WaitGroup

	interval := time.Second / time.Duration(runsPerSecond)

	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	for i := 0; i < runs; i++ {
		<-ticker.C
		wg.Add(1)
		go fn(i, &wg)
	}
	wg.Wait()
}

func RunFuncPerSecondSync(fn func(int), runs, runsPerSecond int) {
	interval := time.Second / time.Duration(runsPerSecond)

	ticker := time.NewTicker(interval)
	defer ticker.Stop()

	for i := 0; i < runs; i++ {
		<-ticker.C
		fn(i)
	}
}

// StartHeartbeatService starts a goroutine that continuously sends heartbeat transactions
func StartHeartbeatService(domain string, intervalSeconds int) {
	connector := substrate.NewSubstrateConnector(true)

	resultsChan := make(chan string, 100)

	nonce := connector.SendHeartbeat(domain, 0, resultsChan)
	fmt.Printf("Started heartbeat service for domain %s with initial nonce: %d\n", domain, nonce)

	<-resultsChan

	ticker := time.NewTicker(time.Duration(intervalSeconds) * time.Second)
	defer ticker.Stop()

	heartbeatCount := 1

	for {
		<-ticker.C

		nonce = connector.SendHeartbeat(domain, nonce, resultsChan)

		<-resultsChan

		fmt.Printf("Sent heartbeat #%d for domain %s at %s\n",
			heartbeatCount,
			domain,
			time.Now().Format(time.RFC3339))

		heartbeatCount++
	}
}
