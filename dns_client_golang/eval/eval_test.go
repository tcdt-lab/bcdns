package eval

import (
	"os"
	"sync"
	"testing"
	"time"
)

// TestWriteToCSV tests the WriteToCSV function
func TestWriteToCSV(t *testing.T) {
	// Create test data
	results := []*EvalResult{
		{Idx: 1, Time: 100},
		{Idx: 2, Time: 200},
		{Idx: 3, Time: 300},
	}
	
	// Create a temporary file for testing
	tempFile := "test_eval.csv"
	defer os.Remove(tempFile) // Clean up after test
	
	// Call the function
	WriteToCSV(tempFile, results)
	
	// Verify the file was created
	_, err := os.Stat(tempFile)
	if err != nil {
		t.Errorf("Failed to create CSV file: %v", err)
	}
}

// TestRunFuncPerSecond tests the RunFuncPerSecond function
func TestRunFuncPerSecond(t *testing.T) {
	// Create a channel to collect results
	resultChan := make(chan int, 5)
	
	// Create a test function
	testFunc := func(i int, wg *sync.WaitGroup) {
		resultChan <- i
		wg.Done()
	}
	
	// Set test parameters
	runs := 2
	runsPerSecond := 10 // Should complete in less than a second
	
	// Call the function
	RunFuncPerSecond(testFunc, runs, runsPerSecond)
	
	// Collect results
	var results []int
	for i := 0; i < runs; i++ {
		select {
		case result := <-resultChan:
			results = append(results, result)
		case <-time.After(100 * time.Millisecond):
			t.Fatalf("Timed out waiting for result %d", i)
		}
	}
	
	// Check if all expected runs were executed
	if len(results) != runs {
		t.Errorf("Expected %d results, got %d", runs, len(results))
	}
}

// TestRunFuncPerSecondSync tests the RunFuncPerSecondSync function
func TestRunFuncPerSecondSync(t *testing.T) {
	// Create a slice to collect results
	var results []int
	
	// Create a test function
	testFunc := func(i int) {
		results = append(results, i)
	}
	
	// Set test parameters
	runs := 2
	runsPerSecond := 10 // Should complete in less than a second
	
	// Call the function
	RunFuncPerSecondSync(testFunc, runs, runsPerSecond)
	
	// Check if all expected runs were executed
	if len(results) != runs {
		t.Errorf("Expected %d results, got %d", runs, len(results))
	}
	
	// Check if the results are in order (since this is the sync version)
	for i := 0; i < runs; i++ {
		if results[i] != i {
			t.Errorf("Expected result at index %d to be %d, got %d", i, i, results[i])
		}
	}
}
