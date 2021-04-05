package mtree

import (
	"container/list"

	"crypto/sha256"
	"github.com/ethereum/go-ethereum/crypto"
)

type DagData Hash

func (dd DagData) Copy() NodeData {
	result := DagData{}
	copy(result[:], dd[:])
	return result
}

type DagTree struct {
	MerkleTree
}

func _sha256(first, second []byte) []byte {
	result := sha256.Sum256(append(first, second...))
	return result[:]
}

// turns a dag element data (128 bytes) into a hash
// by following rules:
// 1. assume data is `abcd` where a, b, c, d are 32 bytes
// 2. `first = concat(reverse(a), reverse(b))`
// 3. `second = concat(reverse(c), reverse(d))`
// 4. `keccak = hash(first, second)`, basically keccak256 over concat(first, second)
// 5. result is the last half of `keccak` because keccak is 32 bytes and our hash is 16 bytes
func _sha256ElementHash(data ElementData) NodeData {
	// insert data into the mtbuf and aggregate the
	// hashes
	// because contract side is expecting the bytes
	// to be reversed each 32 bytes on leaf nodes
	first, second := conventionalWord(data.(Word))
	keccak := _sha256(first, second)
	result := DagData{}
	copy(result[:HashLength], keccak[HashLength:])
	return result
}

func _sha256Hash(a, b NodeData) NodeData {
	var keccak []byte
	left := a.(DagData)
	right := b.(DagData)
	keccak = _sha256(
		append([]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}, left[:]...),
		append([]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}, right[:]...))
	result := DagData{}
	copy(result[:HashLength], keccak[HashLength:])
	return result
}

// turns a dag element data (128 bytes) into a hash
// by following rules:
// 1. assume data is `abcd` where a, b, c, d are 32 bytes
// 2. `first = concat(reverse(a), reverse(b))`
// 3. `second = concat(reverse(c), reverse(d))`
// 4. `keccak = hash(first, second)`, basically keccak256 over concat(first, second)
// 5. result is the last half of `keccak` because keccak is 32 bytes and our hash is 16 bytes
func _elementHash(data ElementData) NodeData {
	// insert data into the mtbuf and aggregate the
	// hashes
	// because contract side is expecting the bytes
	// to be reversed each 32 bytes on leaf nodes
	first, second := conventionalWord(data.(Word))
	keccak := crypto.Keccak256(first, second)
	result := DagData{}
	copy(result[:HashLength], keccak[HashLength:])
	return result
}

func _hash(a, b NodeData) NodeData {
	var keccak []byte
	left := a.(DagData)
	right := b.(DagData)
	keccak = crypto.Keccak256(
		append([]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}, left[:]...),
		append([]byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}, right[:]...),
	)
	result := DagData{}
	copy(result[:HashLength], keccak[HashLength:])
	return result
}

func _modifier(data NodeData) {}

func NewSHA256DagTree() *DagTree {
	mtbuf := list.New()
	return &DagTree{
		MerkleTree{
			mtbuf,
			_sha256Hash,
			_sha256ElementHash,
			_modifier,
			false,
			map[uint32]bool{},
			[]uint32{},
			0,
			0,
			[]NodeData{},
		},
	}
}

func NewKeccak256DagTree() *DagTree {
	mtbuf := list.New()
	return &DagTree{
		MerkleTree{
			mtbuf,
			_hash,
			_elementHash,
			_modifier,
			false,
			map[uint32]bool{},
			[]uint32{},
			0,
			0,
			[]NodeData{},
		},
	}
}

func (dt DagTree) RootHash() Hash {
	if dt.finalized {
		return Hash(dt.Root().(DagData))
	}
	panic("SP Merkle tree needs to be finalized by calling mt.Finalize()")
}

// func (dt DagTree) MerkleNodes() []*big.Int {
// 	if dt.finalized {
// 		result := []*big.Int{}
// 		for i := 0; i*2 < len(dt.exportNodes); i++ {
// 			if i*2+1 >= len(dt.exportNodes) {
// 				result = append(result,
// 					BranchElementFromHash(
// 						Hash(DagData{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}),
// 						Hash(dt.exportNodes[i*2].(DagData))).Big())
// 			} else {
// 				result = append(result,
// 					BranchElementFromHash(
// 						Hash(dt.exportNodes[i*2+1].(DagData)),
// 						Hash(dt.exportNodes[i*2].(DagData))).Big())
// 			}
// 		}
// 		return result
// 	}
// 	panic("SP Merkle tree needs to be finalized by calling mt.Finalize()")
// }

func (dt DagTree) ProofsForRegisteredIndices() [][]Hash {
	if dt.finalized {
		result := [][]Hash{}
		branches := dt.Branches()
		for _, k := range dt.Indices() {
			oneRes := []Hash{}
			hh := branches[k].ToNodeArray()[1:]
			hashes := hh[:len(hh)-int(dt.StoredLevel())]
			for i := 0; i < len(hashes); i++ {
				oneRes = append(oneRes, Hash(hashes[i].(DagData)))
			}
			result = append(result, oneRes)
		}
		return result
	}
	panic("SP Merkle tree needs to be finalized by calling mt.Finalize()")
}

// return only one array with necessary hashes for each
// index in order. Element's hash and root are not included
// eg. registered indexes are 1, 2, each needs 2 hashes
// then the function return an array of 4 hashes [a1, a2, b1, b2]
// where a1, a2 are proof branch for element at index 1
// b1, b2 are proof branch for element at index 2
func (dt DagTree) AllBranchesArray() []BranchElement {
	if dt.finalized {
		result := []BranchElement{}
		branches := dt.Branches()
		for _, k := range dt.Indices() {
			// p := proofs[k]
			// fmt.Printf("Index: %d\nRawData: %s\nHashedData: %s\n", k, hex.EncodeToString(p.RawData[:]), proofs[k].HashedData.Hex())
			hh := branches[k].ToNodeArray()[1:]
			hhs := hh[:len(hh)-int(dt.StoredLevel())]
			hashes := []Hash{}
			for _, h := range hhs {
				hashes = append(hashes, Hash(h.(DagData)))
			}
			result = append(result, HashesToBranchesArray(hashes)...)
		}
		return result
	}
	panic("SP Merkle tree needs to be finalized by calling mt.Finalize()")
}

func (dt DagTree) AllDAGElements() []Word {
	if dt.finalized {
		result := []Word{}
		branches := dt.Branches()
		for _, k := range dt.Indices() {
			// p := branches[k]
			// fmt.Printf("Index: %d\nRawData: %s\nHashedData: %s\n", k, hex.EncodeToString(p.RawData[:]), proofs[k].HashedData.Hex())
			result = append(result, branches[k].RawData.(Word))
		}
		return result
	}
	panic("SP Merkle tree needs to be finalized by calling mt.Finalize()")
}
