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

    // TODO: implement light client
    function addBlockHeaders(bytes[] memory blockHeaders) public onlyOwner {
        uint256 largestBlockNumber = lastBlockNumber;
        for (uint i = 0; i < blockHeaders.length; i++) {
            NearDecoder.BlockHeaderInnerLite memory header = _readExactHeader(blockHeaders[i]);
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
