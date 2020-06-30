pragma solidity ^0.5.0;

import "../../nearbridge/contracts/INearBridge.sol";


contract NearBridgeMock is INearBridge {
    mapping(uint256 => bytes32) public blockHashes;
    mapping(uint256 => bytes32) public blockMerkleRoots;

    function setBlockMerkleRoot(uint256 blockNumber, bytes32 root) external {
        blockMerkleRoots[blockNumber] = root;
    }

    function setBlockHash(uint256 blockNumber, bytes32 hash) external {
        blockHashes[blockNumber] = hash;
    }

    function balanceOf(address /*wallet*/) external view returns(uint256) {
        return 0;
    }

    function deposit() external payable {
    }

    function withdraw() external {
    }

    function initWithBlock(bytes calldata data) external {
    }

    function addLightClientBlock(bytes calldata data) external payable {
    }

    function challenge(address payable receiver, uint256 signatureIndex, bytes calldata data) external {
    }

    function checkBlockProducerSignatureInLastBlock(uint256 signatureIndex, bytes calldata data) external view returns(bool) {
        return true;
    }
}
