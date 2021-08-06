package mtree

import (
	"math/big"

	"github.com/ethereum/go-ethereum/common/hexutil"
)

const (
	HashLength          = 16  // bytes
	WordLength          = 128 // bytes
	BranchElementLength = 32  // bytes
)

type (
	Word          [WordLength]byte
	Hash          [HashLength]byte
	BranchElement [BranchElementLength]byte
)

func (w Word) ToUint256Array() []*big.Int {
	result := []*big.Int{}
	for i := 0; i < WordLength/32; i++ {
		z := big.NewInt(0)
		// reverse the bytes because contract expects
		// big Int is constructed in little endian
		z.SetBytes(rev(w[i*32 : (i+1)*32]))
		result = append(result, z)
	}
	return result
}

func (h Hash) String() string { return hexutil.Encode(h[:]) }
func (h Hash) Bytes() []byte  { return h[:] }
func (h Hash) Big() *big.Int  { return BytesToBig(h[:]) }
func (h Hash) Hex() string    { return hexutil.Encode(h[:]) }

func (h BranchElement) String() string { return hexutil.Encode(h[:]) }
func (h BranchElement) Bytes() []byte  { return h[:] }
func (h BranchElement) Big() *big.Int  { return BytesToBig(h[:]) }
func (h BranchElement) Hex() string    { return hexutil.Encode(h[:]) }

func BranchElementFromHash(a, b Hash) BranchElement {
	result := BranchElement{}
	copy(result[:], append(a[:], b[:]...)[:BranchElementLength])
	return result
}
