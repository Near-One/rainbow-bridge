package ethashproof

import (
	"fmt"
	"os"

	"github.com/tranvictor/ethashproof/ethash"
	"github.com/tranvictor/ethashproof/mtree"
)

func CalculateProof(blockno uint64, index uint32, cache *DatasetMerkleTreeCache) (mtree.Word, []mtree.Hash, error) {
	dt := mtree.NewSHA256DagTree()

	fullSize := ethash.DAGSize(blockno)
	fullSizeIn128Resolution := fullSize / 128
	branchDepth := len(fmt.Sprintf("%b", fullSizeIn128Resolution-1))
	dt.RegisterStoredLevel(uint32(uint64(branchDepth)-CACHE_LEVEL), uint32(0))
	liveLevel := uint64(branchDepth) - CACHE_LEVEL
	subtreeStart := index >> liveLevel << liveLevel
	dt.RegisterIndex(index - subtreeStart)
	path := ethash.PathToDAG(uint64(blockno/30000), ethash.DefaultDir)
	f, err := os.Open(path)
	if err != nil {
		return mtree.Word{}, []mtree.Hash{}, err
	}
	defer f.Close()
	processDuringRead(f, int(subtreeStart), uint32(1<<(uint64(branchDepth)-CACHE_LEVEL)), dt)
	dt.Finalize()
	element := dt.AllDAGElements()[0]
	proof := dt.ProofsForRegisteredIndices()[0]
	cacheIndex := index >> liveLevel
	proof = append(proof, cache.Proofs[cacheIndex]...)
	return element, proof, nil
}
