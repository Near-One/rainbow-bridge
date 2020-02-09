pragma solidity ^0.5.0;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/ownership/Ownable.sol";
import "./NearDecoder.sol";


contract NearBridge is Ownable {

    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;

    uint256 public lastBlockNumber;
    mapping(uint256 => bytes32) public blockHashes;

    event BlockHashAdded(
        uint256 indexed blockNumber,
        bytes32 blockHash
    );

    function addMissingBlocks(bytes[] memory blockHeaders) public {
        bytes32 prevHash;
        for (uint i = 0; i < blockHeaders.length; i++) {
            NearDecoder.BlockHeaderInnerLite memory header = _readExactHeader(blockHeaders[i]);

            bytes32 hash = keccak256(blockHeaders[i]);

            if (i == 0) {
                // Store only first header of chain
                _addBlockHash(header.height, hash);
            } else {
                // Check sequence of hashes
                // TODO:
                // require(header.prevHash == prevHash, "NearBridge: chain is broken");
            }

            prevHash = hash;

            if (i == blockHeaders.length - 1) {
                // Check latest matches
                require(blockHashes[header.height] == prevHash, "NearBridge: latest hash in chain should match existing");
            }
        }
    }

    // TODO: implement light client
    function addBlockHashes(bytes[] memory blockHeaders) public onlyOwner {
        uint256 largestBlockNumber = 0;
        for (uint i = 0; i < blockHeaders.length; i++) {
            NearDecoder.BlockHeaderInnerLite memory header = _readExactHeader(blockHeaders[i]);
            require(header.height > lastBlockNumber, "NearBridge: can't rewrite existing records");
            _addBlockHash(header.height, keccak256(blockHeaders[i]));
            if (header.height > largestBlockNumber) {
                largestBlockNumber = header.height;
            }
        }

        if (largestBlockNumber > lastBlockNumber) {
            lastBlockNumber = largestBlockNumber;
        }
    }

    function _addBlockHash(uint256 blockNumber, bytes32 hash) internal {
        blockHashes[blockNumber] = hash;
        emit BlockHashAdded(blockNumber, hash);
    }

    function _readExactHeader(bytes memory blockHeader) internal pure returns(NearDecoder.BlockHeaderInnerLite memory header) {
        Borsh.Data memory data = Borsh.from(blockHeader);
        header = data.decodeBlockHeaderInnerLite();
        require(data.finished(), "NearBridge: only block header should be passed");
    }
}
