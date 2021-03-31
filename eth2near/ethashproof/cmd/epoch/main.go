package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/user"
	"path/filepath"

	"github.com/tranvictor/ethashproof"
)

func getHomeDir() string {
	usr, err := user.Current()
	if err != nil {
		log.Fatal(err)
	}
	return usr.HomeDir
}

func main() {
	for i := 0; i < 512; i++ {
		os.RemoveAll(filepath.Join(getHomeDir(), ".ethash"))
		fmt.Printf("Calculating merkle root for the epoch %d\n", i)
		root, err := ethashproof.CalculateDatasetMerkleRoot(uint64(i), false)
		if err != nil {
			fmt.Printf("Calculating dataset merkle root failed: %s\n", err)
			return
		}
		os.Stdout.Write([]byte(root.Hex()))
		fmt.Println()
		err = ioutil.WriteFile(
			fmt.Sprintf("%d.txt", i),
			[]byte(root.Hex()),
			0644,
		)
		if err != nil {
			fmt.Printf("Write merkle root to file: %s\n", err)
			return
		}
	}
}
