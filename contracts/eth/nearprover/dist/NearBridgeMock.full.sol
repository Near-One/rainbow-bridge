// File: contracts/bridge/INearBridge.sol

pragma solidity ^0.6;

interface INearBridge {
    event BlockHashAdded(uint64 indexed height, bytes32 blockHash);

    event BlockHashReverted(uint64 indexed height, bytes32 blockHash);

    function blockHashes(uint64 blockNumber) external view returns (bytes32);

    function blockMerkleRoots(uint64 blockNumber) external view returns (bytes32);

    function balanceOf(address wallet) external view returns (uint256);

    function deposit() external payable;

    function withdraw() external;

    function initWithValidators(bytes calldata initialValidators) external;

    function initWithBlock(bytes calldata data) external;

    function addLightClientBlock(bytes calldata data) external;

    function challenge(address payable receiver, uint256 signatureIndex) external;

    function checkBlockProducerSignatureInHead(uint256 signatureIndex) external view returns (bool);
}

// File: contracts/NearBridgeMock.sol

pragma solidity ^0.6;


contract NearBridgeMock is INearBridge {
    mapping(uint64 => bytes32) public override blockHashes;
    mapping(uint64 => bytes32) public override blockMerkleRoots;

    function setBlockMerkleRoot(uint64 blockNumber, bytes32 root) external {
        blockMerkleRoots[blockNumber] = root;
    }

    function setBlockHash(uint64 blockNumber, bytes32 hash) external {
        blockHashes[blockNumber] = hash;
    }

    function balanceOf(address) external view override returns (uint256) {
        return 0;
    }

    function deposit() external payable override {}

    function withdraw() external override {}

    function initWithValidators(bytes calldata) external override {}

    function initWithBlock(bytes calldata) external override {}

    function addLightClientBlock(bytes calldata) external override {}

    function challenge(address payable, uint256) external override {}

    function checkBlockProducerSignatureInHead(uint256) external view override returns (bool) {
        return true;
    }
}
