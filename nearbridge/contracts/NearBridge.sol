pragma solidity ^0.5.0;
pragma experimental ABIEncoderV2; // solium-disable-line no-experimental

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/ownership/Ownable.sol";
import "./NearDecoder.sol";


contract NearBridge is Ownable {

    using SafeMath for uint256;
    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;

    uint256 public lastBlockNumber;
    bytes32 public lastEpochId;
    bytes32 public lastNextEpochId;
    mapping(uint256 => bytes32) public blockHashes;

    event BlockHashAdded(
        uint256 indexed blockNumber,
        bytes32 blockHash
    );

    function addLightClientBlock(bytes memory data) public {
        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh.decodeLightClientBlock();
        require(borsh.finished(), "NearBridge: only light client block should be passed");

        // 1. The height of the block is higher than the height of the current head
        require(
            nearBlock.inner_lite.height > lastBlockNumber,
            "Height of the block is not valid"
        );
        lastBlockNumber = nearBlock.inner_lite.height;

        // 2. The epoch of the block is equal to the epoch_id or next_epoch_id known for the current head
        require(
            nearBlock.inner_lite.epoch_id == lastEpochId || nearBlock.inner_lite.epoch_id == lastNextEpochId,
            "Epoch id of the block is not valid"
        );
        lastEpochId = nearBlock.inner_lite.epoch_id;
        lastNextEpochId = nearBlock.inner_lite.next_epoch_id;

        // 3. If the epoch of the block is equal to the next_epoch_id of the head, then next_bps is not None
        if (nearBlock.inner_lite.epoch_id == lastNextEpochId) {
            require(
                !nearBlock.next_bps.none,
                "Next bps should no be None"
            );
        }

        // 4. approvals_next and approvals_after_next contain signatures that check out against the block producers for the epoch of the block
        // 5. The signatures present in both approvals_next and approvals_after_next correspond to more than 2/3 of the total stake
        uint256 totalStake = 0;
        for (uint i = 0; i < nearBlock.next_bps.validatorStakes.length; i++) {
            totalStake = totalStake.add(
                nearBlock.next_bps.validatorStakes[i].stake
            );
        }
        require(
            _checkValidatorSignatures(
                totalStake,
                nearBlock.next_block_inner_hash,
                nearBlock.approvals_next,
                nearBlock.next_bps.validatorStakes
            ),
            "Less than 2/3 voted for the next block"
        );
        require(
            _checkValidatorSignatures(
                totalStake,
                nearBlock.next_block_inner_hash,
                nearBlock.approvals_after_next,
                nearBlock.next_bps.validatorStakes
            ),
            "Less than 2/3 voted for the block after next"
        );

        // 6. If next_bps is not none, sha256(borsh(next_bps)) corresponds to the next_bp_hash in inner_lite.
        if (!nearBlock.next_bps.none) {
            require(
                nearBlock.next_bps.hash == nearBlock.inner_lite.next_bp_hash,
                "Hash of block producers do not match"
            );
        }

        // Finfish:
        lastBlockNumber = nearBlock.inner_lite.height;
        lastEpochId = nearBlock.inner_lite.epoch_id;
        lastNextEpochId = nearBlock.inner_lite.next_epoch_id;
        blockHashes[nearBlock.inner_lite.height] = hash(nearBlock);
        emit BlockHashAdded(
            lastBlockNumber,
            blockHashes[lastBlockNumber]
        );
    }

    function _checkValidatorSignatures(
        uint256 totalStake,
        bytes32 next_block_inner_hash,
        NearDecoder.OptionalSignature[] memory approvals,
        NearDecoder.ValidatorStake[] memory validatorStakes
    ) internal view returns(bool) {
        uint256 votedFor = 0;
        uint256 votedAgainst = 0;
        for (uint i = 0; i < approvals.length; i++) {
            if (approvals[i].none) {
                votedAgainst = votedAgainst.add(validatorStakes[i].stake);
            } else {
                address publicKeyHashRecovered = ecrecover(
                    next_block_inner_hash,
                    approvals[i].signature.v,
                    approvals[i].signature.r,
                    approvals[i].signature.s
                );
                bytes32 publicKeyHashComputed = keccak256(abi.encodePacked(
                    validatorStakes[i].public_key.x,
                    validatorStakes[i].public_key.y
                ));
                require(
                    publicKeyHashRecovered == address(uint160(bytes20(publicKeyHashComputed))),
                    "Validator signature is not valid"
                );
                votedFor = votedFor.add(validatorStakes[i].stake);
            }

            if (votedFor > totalStake.mul(2).div(3)) {
                return true;
            }
            if (votedAgainst > totalStake.mul(1).div(3)) {
                return false;
            }
        }

        revert("Should never be reached");
    }

    // function addBlockHeaders(bytes[] memory blockHeaders) public onlyOwner {
    //     uint256 largestBlockNumber = lastBlockNumber;
    //     for (uint i = 0; i < blockHeaders.length; i++) {
    //         NearDecoder.BlockHeaderInnerLite memory header = _readExactHeader(blockHeaders[i]);
    //         _addBlockHash(header.height, keccak256(blockHeaders[i]));
    //         if (header.height > largestBlockNumber) {
    //             largestBlockNumber = header.height;
    //         }
    //     }

    //     if (largestBlockNumber > lastBlockNumber) {
    //         lastBlockNumber = largestBlockNumber;
    //     }
    // }

    // function _addBlockHash(uint256 blockNumber, bytes32 hash) internal {
    //     blockHashes[blockNumber] = hash;
    //     emit BlockHashAdded(blockNumber, hash);
    // }

    function hash(NearDecoder.LightClientBlock memory nearBlock) public view returns(bytes32) {
        return keccak256(abi.encodePacked(
            nearBlock.prev_block_hash,
            keccak256(abi.encodePacked(
                nearBlock.inner_lite.hash,
                nearBlock.inner_rest_hash
            ))
        ));
    }
}
