package main

import (
	"fmt"
	"os"
	"strconv"

	"github.com/tranvictor/ethashproof"
)

func usage(msg string) {
	fmt.Printf("Error:" + msg + "\n")
	fmt.Printf("Usage: \t ./cache <epoch_number> or \n\t\t ./cache <epoch_number> <cache_dir>\n")
	os.Exit(1)
}

func getEpoch(idx int) int {
	num, err := strconv.Atoi(os.Args[idx])
	if err != nil {
		usage("Please pass a number as epoch number")
	}
	return num
}

func getCacheDir(idx int) string {
	dir := os.Args[idx]
	err := os.MkdirAll(dir, 0700)
	if err != nil {
		usage("Cannot create cacheDir. Please pass path to cacheDir")
	}
	return dir
}

func main() {
	argsCount := len(os.Args)
	epochArgIdx := 1
	cacheDirArgIdx := 2
        var epoch int
	var cacheDir string

	switch argsCount {
	case 1:
		usage("Error: Epoch number param is missing\n")
	case 2:
		epoch = getEpoch(epochArgIdx)
		cacheDir = "|default|"
	case 3:
		epoch = getEpoch(epochArgIdx)
		cacheDir = getCacheDir(cacheDirArgIdx)
	}

	root, err := ethashproof.CalculateDatasetMerkleRoot(uint64(epoch), true, cacheDir)
	if err != nil {
		fmt.Printf("Calculating dataset merkle root failed: %s\n", err)
		return
	}

	fmt.Printf("Root: %s\n", root.Hex())
}
