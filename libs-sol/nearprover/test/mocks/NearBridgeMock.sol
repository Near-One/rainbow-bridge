pragma solidity ^0.5.0;

import "../../../nearbridge/contracts/INearBridge.sol";


contract NearBridgeMock is INearBridge {
    mapping(uint256 => bytes32) public blockHashes;

    function setBlockHashes(uint256 blockNumber, bytes32 hash) external returns(bytes32) {
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
    }
}
