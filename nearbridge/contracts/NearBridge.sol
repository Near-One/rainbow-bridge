pragma solidity ^0.5.0;

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
    function addBlockHash(bytes memory block) public onlyOwner {
        Borsh.Data memory data = Borsh.Data({
            offset: 0,
            raw: block
        });

        NearDecoder.BlockHeaderInnerLite memory header = data.decodeBlockHeaderInnerLite();
        require(data.finished(), "NearBridge: only block header should be passed");
        require(header.height > lastBlockNumber, "NearBridge: can't rewrite existing records");

        bytes32 hash = keccak256(block);
        blockHashes[header.height] = hash;
        emit BlockHashAdded(header.height, hash);
        lastBlockNumber = header.height;
    }
}
