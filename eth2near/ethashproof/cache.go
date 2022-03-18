package ethashproof

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/user"
	"path/filepath"

	"github.com/tranvictor/ethashproof/mtree"
)

const CACHE_LEVEL uint64 = 15

type DatasetMerkleTreeCache struct {
	Epoch       uint64         `json:"epoch"`
	ProofLength uint64         `json:"proof_length"`
	CacheLength uint64         `json:"cache_length"`
	RootHash    mtree.Hash     `json:"root_hash"`
	Proofs      [][]mtree.Hash `json:"proofs"`
}

func (self *DatasetMerkleTreeCache) Print() {
	fmt.Printf("Epoch: %d\n", self.Epoch)
	fmt.Printf("Merkle root: %s\n", self.RootHash.Hex())
	fmt.Printf("Sub proofs:\n")
	for i, proof := range self.Proofs {
		fmt.Printf("%d. [", i)
		for _, node := range proof {
			fmt.Printf("%s, ", node.Hex())
		}
		fmt.Printf("]\n")
	}
}

func getHomeDir() string {
	usr, err := user.Current()
	if err != nil {
		log.Fatal(err)
	}
	return usr.HomeDir
}

func PersistCache(cache *DatasetMerkleTreeCache) error {
	content, err := json.Marshal(cache)
	if err != nil {
		return err
	}
	dirPath := filepath.Join(getHomeDir(), ".ethashproof")
	err = os.MkdirAll(dirPath, 0777)
	if err != nil {
		return err
	}
	path := filepath.Join(dirPath, fmt.Sprintf("%d.json", cache.Epoch))
	return ioutil.WriteFile(path, content, 0644)
}

func LoadCache(epoch int) (*DatasetMerkleTreeCache, error) {
	path := filepath.Join(getHomeDir(), ".ethashproof", fmt.Sprintf("%d.json", epoch))
	content, err := ioutil.ReadFile(path)
	if err != nil {
		return nil, err
	}
	result := &DatasetMerkleTreeCache{}
	err = json.Unmarshal(content, &result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func PathToCache(epoch uint64) string {
	return filepath.Join(getHomeDir(), ".ethashproof", fmt.Sprintf("%d.json", epoch))
}
