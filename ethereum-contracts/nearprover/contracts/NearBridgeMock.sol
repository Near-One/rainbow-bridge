pragma solidity ^0.6;

import "../../nearbridge/contracts/INearBridge.sol";


contract NearBridgeMock is INearBridge {
    mapping(uint64 => bytes32) override public blockHashes;
    mapping(uint64 => bytes32) override public blockMerkleRoots;

    function setBlockMerkleRoot(uint64 blockNumber, bytes32 root) external {
        blockMerkleRoots[blockNumber] = root;
    }

    function setBlockHash(uint64 blockNumber, bytes32 hash) external {
        blockHashes[blockNumber] = hash;
    }

    function balanceOf(address) override external view returns(uint256) {
        return 0;
    }

    function deposit() override external payable {
    }

    function withdraw() override external {
    }

    function initWithValidators(bytes calldata) override external {
    }

    function initWithBlock(bytes calldata) override external {
    }

    function addLightClientBlock(bytes calldata) override external {
    }

    function challenge(address payable, uint256) override external {
    }

    function checkBlockProducerSignatureInHead(uint256) override external view returns(bool) {
        return true;
    }
}
