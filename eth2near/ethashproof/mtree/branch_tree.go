package mtree

type BranchTree struct {
	RawData    ElementData
	HashedData NodeData
	Root       *BranchNode
}

func (t BranchTree) ToNodeArray() []NodeData {
	return t.Root.ToNodeArray()
}
