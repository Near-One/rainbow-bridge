pragma solidity ^0.7;

// SPDX-License-Identifier: UNLICENSED

contract Verifier {

    // Debug code
    bytes32 root;
    function setRoot(bytes32 newRoot) external {
        root = newRoot;
    }

    // Proof format:
    // 8 bytes: total number of messages n
    // 32*k bytes: k hashes
    // uid: message index i between 0 and n-1
    // https://github.com/ethereum/solidity/issues/8917
    function verify(bytes calldata /*sender*/, bytes calldata /*message*/, uint uid, bytes calldata /*proof*/) external view {
        assembly {
            let proof := add(4, calldataload(0x64))
            let proofPtr := add(proof, 32)
            let n := shr(192, calldataload(proofPtr))
            if iszero(lt(uid, n)) {
                revert(0, 0)
            }
            let n1 := add(n, 1)
            let v := and(shr(1, xor(n1, uid)), or(not(uid), n1))
            let d2 := shl(5, gt(v, 0xffffffff))
            d2 := add(d2, shl(4, gt(shr(d2, v), 0xffff)))
            d2 := add(d2, shl(3, gt(shr(d2, v), 0xff)))
            d2 := add(d2, shl(2, gt(shr(d2, v), 0xf)))
            d2 := add(d2, and(shr(shl(1, shr(d2, v)), 0xffffaa50), 0x3))
            let d0 := add(d2, and(shr(d2, n1), 1))
            let mask := sub(shl(d0, 1), 1)
            let d1 := gt(uid, mask)
            let path := add(and(uid, mask), shl(d0, d1))
            let pathLen := add(add(d2, d1), d0)
            let proofLen := calldataload(proof)
            if iszero(eq(proofLen, add(8, shl(5, pathLen)))) {
                revert(0, 0)
            }
            let proofEnd := add(proofPtr, proofLen)
            let mem := mload(0x40)
            let sender := add(4, calldataload(4))
            let senderLen := calldataload(sender)
            if gt(senderLen, 0xff) {
                revert(0, 0)
            }
            mstore8(mem, senderLen)
            let memPtr := add(mem, 1)
            calldatacopy(memPtr, add(sender, 32), senderLen)
            memPtr := add(memPtr, senderLen)
            let message := add(4, calldataload(0x24))
            let messageLen := calldataload(message)
            calldatacopy(memPtr, add(message, 32), messageLen)
            let hash := keccak256(mem, sub(add(memPtr, messageLen), mem))
            for { let ptr := add(proofPtr, 8) } iszero(eq(ptr, proofEnd)) { ptr := add(ptr, 32) } {
                let b := shl(5, and(path, 1))
                mstore(b, hash)
                mstore(sub(0x20, b), calldataload(ptr))
                hash := keccak256(0, 0x40)
                path := shr(1, path)
            }
            mstore(0, n)
            mstore(0x20, hash)
            if iszero(eq(sload(root.slot), keccak256(0x18, 0x28))) {
                revert(0, 0)
            }
        }
    }
}
