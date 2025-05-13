package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"time"
)

const (
	COM_TLD_SPECS  = "../all_specs/com_tldSpec.json"
	EXAMPLE_SPECS  = "../all_specs/exampleSpec.json"
	WHATEVER_SPECS = "../all_specs/whateverSpec.json"
)

var (
	FILLER_TLDS        = [...]string{"org", "net", "gov", "edu"}
	FILLER_TARGETS     = [...]string{"test", "example2", "example3", "domain", "domain1", "domain2", "domain3", "google", "facebook", "twitter", "instagram", "linkedin", "youtube", "reddit", "tiktok", "snapchat", "whatsapp", "telegram", "signal", "discord", "slack", "microsoft", "apple", "amazon", "netflix", "spotify", "uber", "lyft", "airbnb", "expedia", "tripadvisor", "booking", "priceline", "kayak"}
	DNS_CONFIGURATIONS = [...]ArchOptions{
		{
			tlds:         1,
			networks:     2,
			validators:   2,
			normal_nodes: 2,
		},
	}
	currentConfig *ArchOptions
)

type ArchOptions struct {
	tlds         int
	networks     int
	validators   int
	normal_nodes int
}

type ArchOption func(*ArchOptions)

func withOptions(options *ArchOptions) ArchOption {
	return func(o *ArchOptions) {
		o.tlds = options.tlds
		o.networks = options.networks
		o.validators = options.validators
		o.normal_nodes = options.normal_nodes
	}
}

func launchArch(setters ...ArchOption) {
	o := &ArchOptions{}

	for _, setter := range setters {
		setter(o)
	}

	cmd := exec.Command("./launch_dns_arch.sh",
		"--tld",
		strconv.Itoa(o.tlds),
		"--target",
		strconv.Itoa(o.networks),
		"--validators",
		strconv.Itoa(o.validators),
		"--nodes",
		strconv.Itoa(o.normal_nodes))
	cmd.Dir = "../"
	out, _ := cmd.Output()
	fmt.Printf("launch_dns_arch.sh: %s\n", string(out))
	fmt.Println("Successfully launched DNS architecture.")
}

func cleanArch() {
	cmd := exec.Command("./dns_arch_cleanup.sh")
	cmd.Dir = "../"
	out, _ := cmd.Output()
	fmt.Printf("dns_arch_cleanup.sh: %s\n", string(out))
	fmt.Println("Successfully cleaned up DNS architecture.")
}

func copySHFiles() error {
	files, err := filepath.Glob("*.sh")
	if err != nil {
		return err
	}

	for _, file := range files {
		src, err := os.Open(file)
		if err != nil {
			return err
		}
		defer src.Close()

		dst, err := os.Create(filepath.Join("..", file))
		if err != nil {
			return err
		}
		defer dst.Close()

		_, err = io.Copy(dst, src)
		if err != nil {
			return err
		}

		// Make the file executable
		err = os.Chmod(filepath.Join("..", file), 0755)
		if err != nil {
			return err
		}
	}
	return nil
}

func setupDNSInfo() {
	fmt.Printf("Setting up DNS information...")

	cmd := exec.Command("npm",
		"install")
	cmd.Dir = "../../dns_client"

	var out, err = cmd.Output()

	if err != nil {
		fmt.Printf("Error executing command: %v\n", err)
	}

	cmd = exec.Command("npm",
		"run",
		"register",
		"--",
		"--tld",
		"com",
		"../polkadot-sdk-solochain-template/all_specs/com_tldSpec.json",
		"//Alice")
	cmd.Dir = "../../dns_client"

	out, err = cmd.Output()

	if err != nil {
		fmt.Printf("Error executing command: %v\n", err)
	}

	fmt.Printf("Registered COM TLD. Output: %s\n", out)

	for _, tld := range FILLER_TLDS {
		fillerCmd := exec.Command("npm",
			"run",
			"register",
			"--",
			"--tld",
			tld,
			"../polkadot-sdk-solochain-template/all_specs/com_tldSpec.json", // Can use any value since this is filler information
			"//Alice")
		fillerCmd.Dir = "../../dns_client"

		var fillerOut, fillerErr = fillerCmd.Output()

		if fillerErr != nil {
			fmt.Printf("Error executing command: %v\n", fillerErr)
		}

		fmt.Printf("Registered %s TLD. Output: %s\n", tld, fillerOut)
	}

	time.Sleep(time.Millisecond * 10000) // Wait 10 seconds for ledger to stabilize

	cmd = exec.Command("npm",
		"run",
		"register",
		"--",
		"--domain",
		"example.com",
		"../polkadot-sdk-solochain-template/all_specs/exampleSpec.json",
		"//Alice")
	cmd.Dir = "../../dns_client"

	out, err = cmd.Output()

	if err != nil {
		fmt.Printf("Error executing command: %v\n", err)
	}

	fmt.Printf("Registered example network. Output: %s\n", out)

	cmd = exec.Command("npm",
		"run",
		"register",
		"--",
		"--domain",
		"whatever.com",
		"../polkadot-sdk-solochain-template/all_specs/whateverSpec.json",
		"//Alice")
	cmd.Dir = "../../dns_client"

	out, err = cmd.Output()

	if err != nil {
		fmt.Printf("Error executing command: %v\n", err)
	}

	fmt.Printf("Registered whatever network. Output: %s\n", out)

	for _, target := range FILLER_TARGETS {
		fillerCmd := exec.Command("npm",
			"run",
			"register",
			"--",
			"--domain",
			target+".com",
			"../polkadot-sdk-solochain-template/all_specs/exampleSpec.json", // Can use any value since this is filler information
			"//Alice")
		fillerCmd.Dir = "../../dns_client"

		out, err = fillerCmd.Output()

		if err != nil {
			fmt.Printf("Error executing command: %v\n", err)
		}

		fmt.Printf("Registered %s network. Output: %s\n", target, out)
	}

	time.Sleep(time.Millisecond * 10000) // Wait 10 seconds for ledger to stabilize
}

func main() {
	flagInit := flag.Bool("init", false, "Initialize architecture and setup DNS")
	flagCleanup := flag.Bool("cleanup", false, "Clean up architecture")
	flag.Parse()

	if err := copySHFiles(); err != nil {
		fmt.Printf("Error copying .sh files: %v\n", err)
	}

	for _, o := range DNS_CONFIGURATIONS {
		currentConfig = &o

		switch {
		case *flagInit:
			fmt.Printf("Launching architecture using settings:\n %+v\n", o)
			launchArch(withOptions(&o))
			setupDNSInfo()
		case *flagCleanup:
			fmt.Printf("Cleaning up architecture using settings:\n %+v\n", o)
			cleanArch()
		default:
			fmt.Printf("Launching architecture using settings:\n %+v\n", o)
			launchArch(withOptions(&o))
			setupDNSInfo()
			cleanArch()
		}
	}
}
