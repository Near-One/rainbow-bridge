// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8;

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
