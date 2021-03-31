package mtree

type BranchNode struct {
	Hash             NodeData
	Left             *BranchNode
	Right            *BranchNode
	ElementOnTheLeft bool
}

func (b BranchNode) ToNodeArray() []NodeData {
	if b.Left == nil && b.Right == nil {
		return []NodeData{b.Hash}
	}
	left := b.Left.ToNodeArray()
	right := b.Right.ToNodeArray()
	if b.ElementOnTheLeft {
		return append(left, right...)
	} else {
		return append(right, left...)
	}
}

// explain the operation
func AcceptLeftSibling(b *BranchNode, h NodeData) *BranchNode {
	return &BranchNode{
		Hash:             nil,
		Left:             &BranchNode{h, nil, nil, false},
		Right:            b,
		ElementOnTheLeft: false,
	}
}

func AcceptRightSibling(b *BranchNode, h NodeData) *BranchNode {
	return &BranchNode{
		Hash:             nil,
		Right:            &BranchNode{h, nil, nil, false},
		Left:             b,
		ElementOnTheLeft: true,
	}
}
