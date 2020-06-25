
// File: ../nearbridge/contracts/INearBridge.sol

pragma solidity ^0.5.0;


interface INearBridge {
    event BlockHashAdded(
        uint256 indexed height,
        bytes32 blockHash
    );

    function blockHashes(uint256 blockNumber) external view returns(bytes32);
    function blockMerkleRoots(uint256 blockNumber) external view returns(bytes32);

    function balanceOf(address wallet) external view returns(uint256);
    function deposit() external payable;
    function withdraw() external;

    function initWithBlock(bytes calldata data) external;
    function addLightClientBlock(bytes calldata data) external payable;
    function challenge(address payable receiver, uint256 signatureIndex, bytes calldata data) external;
    function checkBlockProducerSignatureInLastBlock(uint256 signatureIndex, bytes calldata data) external view returns(bool);
}

// File: contracts/NearBridgeMock.sol

pragma solidity ^0.5.0;



contract NearBridgeMock is INearBridge {
    mapping(uint256 => bytes32) public blockMerkleRoots;

    function setBlockMerkleRoot(uint256 blockNumber, bytes32 root) external {
        blockMerkleRoots[blockNumber] = root;
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
