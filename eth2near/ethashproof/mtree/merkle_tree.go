package mtree

import "container/list"

type ElementData interface{}

type NodeData interface {
	Copy() NodeData
}

type node struct {
	Data      NodeData
	NodeCount uint32
	Branches  *map[uint32]BranchTree
}

func (n node) Copy() node {
	return node{n.Data.Copy(), n.NodeCount, &map[uint32]BranchTree{}}
}

type hashFunc func(NodeData, NodeData) NodeData
type elementHashFunc func(ElementData) NodeData
type dummyNodeModifierFunc func(NodeData)

type MerkleTree struct {
	mtbuf           *list.List
	h               hashFunc
	eh              elementHashFunc
	dnf             dummyNodeModifierFunc
	finalized       bool
	indexes         map[uint32]bool
	orderedIndexes  []uint32
	storedLevel     uint32
	exportNodeCount uint32
	exportNodes     []NodeData
}

func (mt *MerkleTree) StoredLevel() uint32 {
	return mt.storedLevel
}

func (mt *MerkleTree) RegisterStoredLevel(depth, level uint32) {
	mt.storedLevel = level
	mt.exportNodeCount = 1<<(depth-level+1) - 1
}

// register indexes to build branches
func (mt *MerkleTree) RegisterIndex(indexes ...uint32) {
	for _, i := range indexes {
		mt.indexes[i] = true
		mt.orderedIndexes = append(mt.orderedIndexes, i)
	}
}

func (mt *MerkleTree) SetHashFunction(_h hashFunc) {
	mt.h = _h
}

func (mt *MerkleTree) SetElementHashFunction(_h elementHashFunc) {
	mt.eh = _h
}

func (mt *MerkleTree) Insert(data ElementData, index uint32) {
	_node := node{mt.eh(data), 1, &map[uint32]BranchTree{}}
	// fmt.Printf("Inserted node for word (%s): %4s\n", hex.EncodeToString(data[:]), hex.EncodeToString(_node.Data[:]))
	if mt.indexes[index] {
		(*_node.Branches)[index] = BranchTree{
			RawData:    data,
			HashedData: _node.Data,
			Root: &BranchNode{
				Hash:  _node.Data,
				Left:  nil,
				Right: nil,
			},
		}
	}
	mt.insertNode(_node)
}

func (mt *MerkleTree) insertNode(_node node) {
	var e, prev *list.Element
	var cNode, prevNode node
	e = mt.mtbuf.PushBack(_node)
	for {
		prev = e.Prev()
		cNode = e.Value.(node)
		if prev == nil {
			break
		}
		prevNode = prev.Value.(node)
		if cNode.NodeCount != prevNode.NodeCount {
			break
		}
		if prevNode.Branches != nil {
			// fmt.Printf("Accepting right sibling\n")
			for k, v := range *prevNode.Branches {
				v.Root = AcceptRightSibling(v.Root, cNode.Data)
				(*prevNode.Branches)[k] = v
				// fmt.Printf("Proof: %v\n", v.String())
			}
		}
		if cNode.Branches != nil {
			// fmt.Printf("Accepting left sibling\n")
			for k, v := range *cNode.Branches {
				v.Root = AcceptLeftSibling(v.Root, prevNode.Data)
				(*prevNode.Branches)[k] = v
				// fmt.Printf("Proof: %v\n", v.String())
			}
		}
		// fmt.Printf("Creating new Node: h(%4s, %4s) ", hex.EncodeToString(prevNode.Data[:]), hex.EncodeToString(cNode.Data[:]))
		prevNode.Data = mt.h(prevNode.Data, cNode.Data)
		// fmt.Printf("=> %4s\n", hex.EncodeToString(prevNode.Data[:]))
		prevNode.NodeCount = cNode.NodeCount*2 + 1
		if prevNode.NodeCount == mt.exportNodeCount {
			mt.exportNodes = append(mt.exportNodes, prevNode.Data)
		}

		mt.mtbuf.Remove(e)
		mt.mtbuf.Remove(prev)
		e = mt.mtbuf.PushBack(prevNode)
	}
}

func (mt *MerkleTree) Finalize() {
	if !mt.finalized && mt.mtbuf.Len() > 1 {
		for {
			dupNode := mt.mtbuf.Back().Value.(node).Copy()
			mt.dnf(dupNode.Data)
			mt.insertNode(dupNode)
			if mt.mtbuf.Len() == 1 {
				break
			}
		}
	}
	mt.finalized = true
}

func (mt MerkleTree) Root() NodeData {
	if mt.finalized {
		return mt.mtbuf.Front().Value.(node).Data
	}
	panic("Merkle tree needs to be finalized by calling mt.Finalize()")
}

func (mt MerkleTree) ExportNodes() []NodeData {
	return mt.exportNodes
}

func (mt MerkleTree) Branches() map[uint32]BranchTree {
	if mt.finalized {
		return *(mt.mtbuf.Front().Value.(node).Branches)
	}
	panic("Merkle tree needs to be finalized by calling mt.Finalize()")
}

func (mt MerkleTree) Indices() []uint32 {
	return mt.orderedIndexes
}
