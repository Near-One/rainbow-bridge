pragma solidity ^0.5.0;
pragma experimental ABIEncoderV2; // solium-disable-line no-experimental

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/ownership/Ownable.sol";
import "./INearBridge.sol";
import "./NearDecoder.sol";
import "./Ed25519.sol";


contract NearBridge is INearBridge {
    using SafeMath for uint256;
    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;

    struct BlockProducer {
        NearDecoder.PublicKey publicKey;
        uint128 stake;
    }

    struct State {
        uint64 height;
        bytes32 epochId;
        bytes32 nextEpochId;
        address submitter;
        uint256 validAfter;
        bytes32 hash;
        bytes32 next_hash;
        uint256 approvals_after_next_length;
        mapping(uint256 => NearDecoder.OptionalSignature) approvals_after_next;

        uint256 next_bps_length;
        uint256 next_total_stake;
        mapping(uint256 => BlockProducer) next_bps;
    }

    bool public initialized;
    uint256 public lock_eth_amount;
    uint256 public lock_duration;
    Ed25519 edwards;
    State public last;
    State public prev;
    State public backup;
    mapping(uint64 => bytes32) public blockHashes;
    mapping(uint64 => bytes32) public blockMerkleRoots;
    mapping(address => uint256) public balanceOf;

    event BlockHashAdded(
        uint64 indexed height,
        bytes32 blockHash
    );

    event BlockHashReverted(
        uint64 indexed height,
        bytes32 blockHash
    );

    constructor(Ed25519 ed, uint256 l_eth, uint256 l_dur) public {
        edwards = ed;
        lock_eth_amount = l_eth;
        lock_duration = l_dur;
    }

    function deposit() public payable {
        require(msg.value == lock_eth_amount && balanceOf[msg.sender] == 0);
        balanceOf[msg.sender] = balanceOf[msg.sender].add(msg.value);
    }

    function withdraw() public {
        require(msg.sender != last.submitter || block.timestamp > last.validAfter);
        balanceOf[msg.sender] = balanceOf[msg.sender].sub(lock_eth_amount);
        msg.sender.transfer(lock_eth_amount);
    }

    function challenge(address payable receiver, uint256 signatureIndex) public {
        require(block.timestamp < last.validAfter, "Lock period already passed");

        require(
            !checkBlockProducerSignatureInLastBlock(signatureIndex),
            "Can't challenge valid signature"
        );

        _payRewardAndRollBack(receiver);
    }

    function checkBlockProducerSignatureInLastBlock(uint256 signatureIndex) public view returns(bool) {
        if (last.approvals_after_next[signatureIndex].none) {
            return true;
        }
        return _checkValidatorSignature(
            last.height,
            last.next_hash,
            last.approvals_after_next[signatureIndex].signature,
            prev.next_bps[signatureIndex].publicKey
        );
    }

    function _payRewardAndRollBack(address payable receiver) internal {
        // Pay reward
        balanceOf[last.submitter] = balanceOf[last.submitter].sub(lock_eth_amount);
        receiver.transfer(lock_eth_amount);

        // Restore last state from backup
        delete blockHashes[last.height];
        delete blockMerkleRoots[last.height];

        emit BlockHashReverted(
            last.height,
            blockHashes[last.height]
        );

        last = backup;
        for (uint i = 0; i < last.next_bps_length; i++) {
            last.next_bps[i] = backup.next_bps[i];
        }
        for (uint i = 0; i < last.approvals_after_next_length; i++) {
            last.approvals_after_next[i] = backup.approvals_after_next[i];
        }
    }

    function initWithBlock(bytes memory data, bytes memory initial_validators) public {
        require(!initialized, "NearBridge: already initialized");
        initialized = true;

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh.decodeLightClientBlock();

        Borsh.Data memory initial_validators_borsh = Borsh.from(initial_validators);
        NearDecoder.InitialValidators memory initialValidators = initial_validators_borsh.decodeInitialValidators();

        require(borsh.finished(), "NearBridge: only light client block should be passed as first argument");
        require(initial_validators_borsh.finished(), "NearBridge: only initial validators should be passed as second argument");

        // Set prev's next_bps to be initialValidators so addLightClientBlock know current epoch's bps to verify
        prev.next_bps_length = initialValidators.validator_stakes.length;
        uint256 totalStake = 0;
        for (uint i = 0; i < initialValidators.validator_stakes.length; i++) {
            prev.next_bps[i] = BlockProducer({
                publicKey: initialValidators.validator_stakes[i].public_key,
                stake: initialValidators.validator_stakes[i].stake
            });
            // Compute total stake
            totalStake = totalStake.add(initialValidators.validator_stakes[i].stake);
        }
        prev.next_total_stake = totalStake;
        _updateBlock(nearBlock, data, true);
    }

    function _checkBp(NearDecoder.LightClientBlock memory nearBlock, State storage prevEpochBlock) internal {
        require(nearBlock.approvals_after_next.length == prevEpochBlock.next_bps_length, "NearBridge: number of BPs should match number of approvals");

        uint256 votedFor = 0;
        for (uint i = 0; i < nearBlock.approvals_after_next.length; i++) {
            if (!nearBlock.approvals_after_next[i].none) {
                // Assume presented signatures are valid, but this could be challenged
                votedFor = votedFor.add(prevEpochBlock.next_bps[i].stake);
            }
        }

        require(votedFor > prevEpochBlock.next_total_stake.mul(2).div(3), "NearBridge: Less than 2/3 voted by the block after next");
    }

    function addLightClientBlock(bytes memory data) public payable {
        require(balanceOf[msg.sender] >= lock_eth_amount, "Balance is not enough");
        require(block.timestamp >= last.validAfter, "Wait until last block become valid");

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh.decodeLightClientBlock();
        require(borsh.finished(), "NearBridge: only light client block should be passed");

        // 1. The height of the block is higher than the height of the current head
        require(
            nearBlock.inner_lite.height > last.height,
            "NearBridge: Height of the block is not valid"
        );

        // 2. The epoch of the block is equal to the epoch_id or next_epoch_id known for the current head
        require(
            nearBlock.inner_lite.epoch_id == last.epochId || nearBlock.inner_lite.epoch_id == last.nextEpochId,
            "NearBridge: Epoch id of the block is not valid"
        );

        // 3. If the epoch of the block is equal to the next_epoch_id of the head, then next_bps is not None
        if (nearBlock.inner_lite.epoch_id == last.nextEpochId) {
            require(
                !nearBlock.next_bps.none,
                "NearBridge: Next next_bps should no be None"
            );
        }

        // 4. approvals_after_next contain signatures that check out against the block producers for the epoch of the block
        // 5. The signatures present in approvals_after_next correspond to more than 2/3 of the total stake
        if (nearBlock.inner_lite.epoch_id == last.epochId) {
            _checkBp(nearBlock, prev);
        } else {
            _checkBp(nearBlock, last);
        }

        // 6. If next_bps is not none, sha256(borsh(next_bps)) corresponds to the next_bp_hash in inner_lite.
        if (!nearBlock.next_bps.none) {
            require(
                nearBlock.next_bps.hash == nearBlock.inner_lite.next_bp_hash,
                "NearBridge: Hash of block producers do not match"
            );
        }

        _updateBlock(nearBlock, data, false);
    }

    function _updateBlock(NearDecoder.LightClientBlock memory nearBlock, bytes memory data, bool init) internal {
        backup = last;
        for (uint i = 0; i < backup.next_bps_length; i++) {
            backup.next_bps[i] = last.next_bps[i];
        }
        for (uint i = 0; i < backup.approvals_after_next.length; i++) {
            backup.approvals_after_next[i] = last.approvals_after_next[i];
        }

        // If next epoch
        if (nearBlock.inner_lite.epoch_id == last.nextEpochId) {
            prev = last;
            for (uint i = 0; i < prev.next_bps_length; i++) {
                prev.next_bps[i] = last.next_bps[i];
            }
        }

        // Compute total stake
        uint256 totalStake = 0;
        for (uint i = 0; i < nearBlock.next_bps.validatorStakes.length; i++) {
            totalStake = totalStake.add(nearBlock.next_bps.validatorStakes[i].stake);
        }

        // Update last
        last = State({
            height: nearBlock.inner_lite.height,
            epochId: nearBlock.inner_lite.epoch_id,
            nextEpochId: nearBlock.inner_lite.next_epoch_id,
            submitter: msg.sender,
            validAfter: init ? 0 : block.timestamp.add(lock_duration),
            hash: keccak256(data),
            next_hash: nearBlock.next_hash,
            approvals_after_next_length: nearBlock.approvals_after_next.length,
            next_bps_length: nearBlock.next_bps.validatorStakes.length,
            next_total_stake: totalStake
        });

        for (uint i = 0; i < nearBlock.next_bps.validatorStakes.length; i++) {
            last.next_bps[i] = BlockProducer({
                publicKey: nearBlock.next_bps.validatorStakes[i].public_key,
                stake: nearBlock.next_bps.validatorStakes[i].stake
            });
        }

        for (uint i = 0; i < nearBlock.approvals_after_next.length; i++) {
            last.approvals_after_next[i] = nearBlock.approvals_after_next[i];
        }

        blockHashes[nearBlock.inner_lite.height] = nearBlock.hash;
        blockMerkleRoots[nearBlock.inner_lite.height] = nearBlock.inner_lite.block_merkle_root;

        emit BlockHashAdded(
            last.height,
            blockHashes[last.height]
        );
    }

    function _checkValidatorSignature(
        uint64 height,
        bytes32 next_block_hash,
        NearDecoder.Signature memory signature,
        NearDecoder.PublicKey storage publicKey
    ) internal view returns(bool) {
        bytes memory message = abi.encodePacked(uint8(0), next_block_hash, _reversedUint64(height + 2), bytes23(0));

        if (signature.enumIndex == 0) {
            (bytes32 arg1, bytes9 arg2) = abi.decode(message, (bytes32, bytes9));
            return publicKey.ed25519.xy != bytes32(0) && edwards.check(
                publicKey.ed25519.xy,
                signature.ed25519.rs[0],
                signature.ed25519.rs[1],
                arg1,
                arg2
            );
        }
        else {
            return ecrecover(
                keccak256(message),
                signature.secp256k1.v + (signature.secp256k1.v < 27 ? 27 : 0),
                signature.secp256k1.r,
                signature.secp256k1.s
                ) == address(uint256(keccak256(abi.encodePacked(
                publicKey.secp256k1.x,
                publicKey.secp256k1.y
            ))));
        }
    }

    function _reversedUint64(uint64 data) private pure returns(uint64 r) {
        r = data;
        r = ((r & 0x00000000FFFFFFFF) << 32) |
            ((r & 0xFFFFFFFF00000000) >> 32);
        r = ((r & 0x0000FFFF0000FFFF) << 16) |
            ((r & 0xFFFF0000FFFF0000) >> 16);
        r = ((r & 0x00FF00FF00FF00FF) << 8) |
            ((r & 0xFF00FF00FF00FF00) >> 8);
    }
}
