package main

import (
	"fmt"
	"os"
	"strconv"

	"github.com/tranvictor/ethashproof"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Printf("Epoch number param is missing. Please run ./cache <epoch_number> instead.\n")
		return
	}
	if len(os.Args) > 3 {
		fmt.Printf("Please run ./cache <epoch_number> instead with available option -f\n")
		return
	}

	forceOption := false
	if len(os.Args) == 3 {
		forceOption = os.Args[3] == "-f"
	}

	epoch, err := strconv.Atoi(os.Args[1])
	if err != nil {
		fmt.Printf("Please pass a number as epoch number. Please run ./cache <integer> instead.\n")
		fmt.Printf("Error: %s\n", err)
		return
	}

	if !forceOption {
		cache, err := ethashproof.LoadCache(int(epoch))
		if err == nil {
			fmt.Printf("Root: %s\n", cache.RootHash.Hex())
			return
		}
	}

	root, err := ethashproof.CalculateDatasetMerkleRoot(uint64(epoch), true)
	if err != nil {
		fmt.Printf("Calculating dataset merkle root failed: %s\n", err)
		return
	}
	fmt.Printf("Root: %s\n", root.Hex())
}
