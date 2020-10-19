// File: ../nearbridge/contracts/INearBridge.sol

pragma solidity ^0.6;


interface INearBridge {
    event BlockHashAdded(
        uint64 indexed height,
        bytes32 blockHash
    );

    event BlockHashReverted(
        uint64 indexed height,
        bytes32 blockHash
    );

    function blockHashes(uint64 blockNumber) external view returns(bytes32);
    function blockMerkleRoots(uint64 blockNumber) external view returns(bytes32);

    function balanceOf(address wallet) external view returns(uint256);
    function deposit() external payable;
    function withdraw() external;

    function initWithValidators(bytes calldata initialValidators) external;
    function initWithBlock(bytes calldata data) external;
    function addLightClientBlock(bytes calldata data) external;
    function challenge(address payable receiver, uint256 signatureIndex) external;
    function checkBlockProducerSignatureInHead(uint256 signatureIndex) external view returns(bool);
}

// File: contracts/NearBridgeMock.sol

pragma solidity ^0.6;



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
