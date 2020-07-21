pragma solidity ^0.5.0;


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

    function initWithBlock(bytes calldata data, bytes calldata initialValidators) external;
    function addLightClientBlock(bytes calldata data) external payable;
    function challenge(address payable receiver, uint256 signatureIndex) external;
    function checkBlockProducerSignatureInLastBlock(uint256 signatureIndex) external view returns(bool);
}
