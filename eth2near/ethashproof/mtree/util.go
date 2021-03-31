package mtree

import (
	"math/big"
)

func HashesToBranchesArray(hashes []Hash) []BranchElement {
	result := []BranchElement{}
	for i := 0; i < len(hashes); i++ {
		result = append(result,
			BranchElementFromHash(
				Hash(DagData{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}),
				hashes[i]))
	}
	return result
}

func BytesToBig(data []byte) *big.Int {
	n := new(big.Int)
	n.SetBytes(data)

	return n
}

func conventionalWord(data Word) ([]byte, []byte) {
	first := rev(data[:32])
	first = append(first, rev(data[32:64])...)
	second := rev(data[64:96])
	second = append(second, rev(data[96:128])...)
	return first, second
}

func rev(b []byte) []byte {
	for i, j := 0, len(b)-1; i < j; i, j = i+1, j-1 {
		b[i], b[j] = b[j], b[i]
	}
	return b
}

func msbPadding(a []byte, size uint32) []byte {
	result := make([]byte, len(a))
	copy(result, a)
	for i := uint32(len(a)); i < size; i++ {
		result = append([]byte{0}, result...)
	}
	return result
}
