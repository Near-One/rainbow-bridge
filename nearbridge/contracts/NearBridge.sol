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

    // Information about the block producers of a certain epoch.
    struct BlockProducerInfo {
        uint256 bpsLength;
        uint256 totalStake;
        mapping(uint256 => BlockProducer) bps;
    }

    // Minimal information about the submitted block.
    struct BlockInfo {
        uint64 height;
        bytes32 epochId;
        bytes32 nextEpochId;
        address submitter;
        uint256 validAfter;
        bytes32 hash;
        bytes32 next_hash;
        uint256 approvals_after_next_length;
        mapping(uint256 => NearDecoder.OptionalSignature) approvals_after_next;
    }

    // Whether the contract was initialized.
    bool public initialized;
    // The `0` address where we are going to send half of the bond when challenge is successful.
    address payable burner;
    uint256 public lockEthAmount;
    uint256 public lockDuration;
    Ed25519 edwards;

    // Block producers of the current epoch.
    BlockProducerInfo public currentBlockProducers;
    // Block producers of the next epoch.
    BlockProducerInfo public nextBlockProducers;
    // Backup current block producers. When candidate block is submitted and it comes from the next epoch, we backup
    // current block producers. Then if it gets successfully challenged, we recover it the following way:
    // nextBlockProducers <- currentBlockProducers
    // currentBlockProducers <- backupCurrentBlockProducers
    BlockProducerInfo public backupCurrentBlockProducers;

    // The most recent block.
    BlockInfo public head;
    // The backup of the previous most recent block, in case it was challenged.
    BlockInfo public backupHead;

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

    constructor(Ed25519 ed, uint256 _lockEthAmount, uint256 _lockDuration) public {
        edwards = ed;
        lockEthAmount = _lockEthAmount;
        lockDuration = _lockDuration;
        burner = address(0);
    }

    function deposit() public payable {
        require(msg.value == lockEthAmount && balanceOf[msg.sender] == 0);
        balanceOf[msg.sender] = balanceOf[msg.sender].add(msg.value);
    }

    function withdraw() public {
        require(msg.sender != head.submitter || block.timestamp > head.validAfter);
        balanceOf[msg.sender] = balanceOf[msg.sender].sub(lockEthAmount);
        msg.sender.transfer(lockEthAmount);
    }

    function challenge(address payable receiver, uint256 signatureIndex) public {
        require(block.timestamp < head.validAfter, "Lock period already passed");

        require(
            !checkBlockProducerSignatureInHead(signatureIndex),
            "Can't challenge valid signature"
        );

        _payRewardAndRollBack(receiver);
    }

    function checkBlockProducerSignatureInHead(uint256 signatureIndex) public view returns(bool) {
        if (head.approvals_after_next[signatureIndex].none) {
            return true;
        }
        return _checkValidatorSignature(
            head.height,
            head.next_hash,
            head.approvals_after_next[signatureIndex].signature,
            currentBlockProducers.bps[signatureIndex].publicKey
        );
    }

    function _payRewardAndRollBack(address payable receiver) internal {
        // Pay reward
        balanceOf[head.submitter] = balanceOf[head.submitter].sub(lockEthAmount);
        receiver.transfer(lockEthAmount / 2);
        burner.transfer(lockEthAmount - lockEthAmount / 2);

        emit BlockHashReverted(
            head.height,
            blockHashes[head.height]
        );

        // Restore last state from backup
        delete blockHashes[head.height];
        delete blockMerkleRoots[head.height];

        if (head.epochId != backupHead.epochId) {
            // When epoch id is different we need to modify the backed up block producers.
            // nextBlockProducers <- currentBlockProducers
            nextBlockProducers = currentBlockProducers;
            for (uint i = 0; i < nextBlockProducers.bpsLength; i++) {
                nextBlockProducers.bps[i] = currentBlockProducers.bps[i];
            }
            // currentBlockProducers <- backupCurrentBlockProducers
            currentBlockProducers = backupCurrentBlockProducers;
            for (uint i = 0; i < currentBlockProducers.bpsLength; i++) {
                currentBlockProducers.bps[i] = backupCurrentBlockProducers.bps[i];
            }
        }

        // Finally we restore the head.
        head = backupHead;
        for (uint i = 0; i < head.approvals_after_next_length; i++) {
            head.approvals_after_next[i] = backupHead.approvals_after_next[i];
        }
    }

    // The first part of initialization -- setting the validators of the current epoch.
    function initWithValidators(bytes memory _initialValidators) public {
        require(!initialized, "NearBridge: already initialized");
        Borsh.Data memory initialValidatorsBorsh = Borsh.from(_initialValidators);
        NearDecoder.InitialValidators memory initialValidators = initialValidatorsBorsh.decodeInitialValidators();
        require(initialValidatorsBorsh.finished(), "NearBridge: only initial validators should be passed as second argument");

        // Set current block producers.
        currentBlockProducers.bpsLength = initialValidators.validator_stakes.length;
        uint256 totalStake = 0;
        for (uint i = 0; i < initialValidators.validator_stakes.length; i++) {
            currentBlockProducers.bps[i] = BlockProducer({
                publicKey: initialValidators.validator_stakes[i].public_key,
                stake: initialValidators.validator_stakes[i].stake
                });
            // Compute total stake
            totalStake = totalStake.add(initialValidators.validator_stakes[i].stake);
        }
        currentBlockProducers.totalStake = totalStake;
    }

    // The second part of the initialization -- setting the current head.
    function initWithBlock(bytes memory data) public {
        require(currentBlockProducers.totalStake > 0, "NearBridge: validators need to be initialized first");
        require(!initialized, "NearBridge: already initialized");
        initialized = true;

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh.decodeLightClientBlock();
        require(borsh.finished(), "NearBridge: only light client block should be passed as first argument");
        _setHead(nearBlock, data, true);
    }

    function _checkBp(NearDecoder.LightClientBlock memory nearBlock, BlockProducerInfo storage bpInfo) internal {
        require(nearBlock.approvals_after_next.length >= bpInfo.bpsLength, "NearBridge: number of approvals should be at least as large as number of BPs");

        uint256 votedFor = 0;
        for (uint i = 0; i < bpInfo.bpsLength; i++) {
            if (!nearBlock.approvals_after_next[i].none) {
                // Assume presented signatures are valid, but this could be challenged
                votedFor = votedFor.add(bpInfo.bps[i].stake);
            }
        }
        // Last block in the epoch might contain extra approvals that light client can ignore.

        require(votedFor > bpInfo.totalStake.mul(2).div(3), "NearBridge: Less than 2/3 voted by the block after next");
    }

    function addLightClientBlock(bytes memory data) public payable {
        require(initialized, "NearBridge: Contract is not initialized.");
        require(balanceOf[msg.sender] >= lockEthAmount, "Balance is not enough");
        require(block.timestamp >= head.validAfter, "Wait until last block become valid");

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh.decodeLightClientBlock();
        require(borsh.finished(), "NearBridge: only light client block should be passed");

        // 1. The height of the block is higher than the height of the current head
        require(
            nearBlock.inner_lite.height > head.height,
            "NearBridge: Height of the block is not valid"
        );

        // 2. The epoch of the block is equal to the epoch_id or next_epoch_id known for the current head
        require(
            nearBlock.inner_lite.epoch_id == head.epochId || nearBlock.inner_lite.epoch_id == head.nextEpochId,
            "NearBridge: Epoch id of the block is not valid"
        );

        // 3. If the epoch of the block is equal to the next_epoch_id of the head, then next_bps is not None
        if (nearBlock.inner_lite.epoch_id == head.nextEpochId) {
            require(
                !nearBlock.next_bps.none,
                "NearBridge: Next next_bps should no be None"
            );
        }

        // 4. approvals_after_next contain signatures that check out against the block producers for the epoch of the block
        // 5. The signatures present in approvals_after_next correspond to more than 2/3 of the total stake
        if (nearBlock.inner_lite.epoch_id == head.epochId) {
            // The new block is from the current epoch.
            _checkBp(nearBlock, currentBlockProducers);
        } else {
            // The new block is from the next epoch.
            _checkBp(nearBlock, nextBlockProducers);
        }

        // 6. If next_bps is not none, sha256(borsh(next_bps)) corresponds to the next_bp_hash in inner_lite.
        if (!nearBlock.next_bps.none) {
            require(
                nearBlock.next_bps.hash == nearBlock.inner_lite.next_bp_hash,
                "NearBridge: Hash of block producers do not match"
            );
        }

        _setHead(nearBlock, data, false);
    }

    function _setHead(NearDecoder.LightClientBlock memory nearBlock, bytes memory data, bool init) internal {
        // If block is from the next epoch or it is initialization then update block producers.
        if (init || nearBlock.inner_lite.epoch_id == head.nextEpochId) {
            // If block from the next epoch then it should contain next_bps.
            require(!nearBlock.next_bps.none, "NearBridge: The first block of the epoch should contain next_bps.");
            // If this is initialization then no need for the backup.
            if (!init) {
                // backupCurrentBlockProducers <- currentBlockProducers
                backupCurrentBlockProducers = currentBlockProducers;
                for (uint i = 0; i < backupCurrentBlockProducers.bpsLength; i++) {
                    backupCurrentBlockProducers.bps[i] = currentBlockProducers.bps[i];
                }
                // currentBlockProducers <- nextBlockProducers
                currentBlockProducers = nextBlockProducers;
                for (uint i = 0; i < currentBlockProducers.bpsLength; i++) {
                    currentBlockProducers.bps[i] = nextBlockProducers.bps[i];
                }
            }
            // nextBlockProducers <- new block producers
            nextBlockProducers.bpsLength = nearBlock.next_bps.validatorStakes.length;
            uint256 totalStake = 0;
            for (uint i = 0; i < nextBlockProducers.bpsLength; i++) {
                nextBlockProducers.bps[i] = BlockProducer({
                    publicKey: nearBlock.next_bps.validatorStakes[i].public_key,
                    stake: nearBlock.next_bps.validatorStakes[i].stake
                    });
                totalStake = totalStake.add(nearBlock.next_bps.validatorStakes[i].stake);
            }
            nextBlockProducers.totalStake = totalStake;
        }

        if (!init) {
            // Backup the head. No need to backup if it is initialization.
            backupHead = head;
            for (uint i = 0; i < head.approvals_after_next_length; i++) {
                backupHead.approvals_after_next[i] = head.approvals_after_next[i];
            }
        }

        // Update the head.
        head = BlockInfo({
            height: nearBlock.inner_lite.height,
            epochId: nearBlock.inner_lite.epoch_id,
            nextEpochId: nearBlock.inner_lite.next_epoch_id,
            submitter: msg.sender,
            validAfter: init ? 0 : block.timestamp.add(lockDuration),
            hash: keccak256(data),
            next_hash: nearBlock.next_hash,
            approvals_after_next_length: nearBlock.approvals_after_next.length
        });
        for (uint i = 0; i < nearBlock.approvals_after_next.length; i++) {
            head.approvals_after_next[i] = nearBlock.approvals_after_next[i];
        }

        blockHashes[nearBlock.inner_lite.height] = nearBlock.hash;
        blockMerkleRoots[nearBlock.inner_lite.height] = nearBlock.inner_lite.block_merkle_root;

        emit BlockHashAdded(
            nearBlock.inner_lite.height,
            blockHashes[nearBlock.inner_lite.height]
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
