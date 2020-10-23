pragma solidity ^0.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/math/SafeMath.sol";
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
        uint256 timestamp;
        bytes32 epochId;
        bytes32 nextEpochId;
        bytes32 hash;
        bytes32 merkleRoot;
        bytes32 next_hash;
    }

    // Whether the contract was initialized.
    bool public initialized;
    // The `0` address where we are going to send half of the bond when challenge is successful.
    address payable burner;
    uint256 public lockEthAmount;
    uint256 public lockDuration;
    // replaceDuration is in nanoseconds, because it is a difference between NEAR timestamps.
    uint256 public replaceDuration;
    Ed25519 edwards;

    // Block producers of the current epoch.
    BlockProducerInfo currentBlockProducers;
    // Block producers of the next epoch.
    BlockProducerInfo nextBlockProducers;

    // The most recent head that is guaranteed to be valid.
    BlockInfo head;

    // The most recently added block. May still be in its challenge period, so should not be trusted.
    BlockInfo untrustedHead;
    // Approvals on the block following untrustedHead.
    uint untrustedApprovalCount;
    mapping (uint => NearDecoder.OptionalSignature) untrustedApprovals;
    // True if untrustedHead is from the following epoch of currentHead.
    // False if it is from the same epoch.
    bool untrustedHeadIsFromNextEpoch;
    // Next block producers from untrustedHead. This variable is meaningful if untrustedHeadIsFromNextEpoch is true.
    BlockProducerInfo untrustedNextBlockProducers;
    // Address of the account which submitted the last block.
    address lastSubmitter;
    // End of challenge period. If zero, untrusted* fields and lastSubmitter are not meaningful.
    uint lastValidAt;

    mapping(uint64 => bytes32) blockHashes_;
    mapping(uint64 => bytes32) blockMerkleRoots_;
    mapping(address => uint256) override public balanceOf;

    event BlockHashAdded(
        uint64 indexed height,
        bytes32 blockHash
    );

    event BlockHashReverted(
        uint64 indexed height,
        bytes32 blockHash
    );

    constructor(Ed25519 ed, uint256 lockEthAmount_, uint256 lockDuration_, uint256 replaceDuration_) public {
        edwards = ed;
        lockEthAmount = lockEthAmount_;
        lockDuration = lockDuration_;
        replaceDuration = replaceDuration_;
        burner = address(0);
    }

    function deposit() override public payable {
        require(msg.value == lockEthAmount && balanceOf[msg.sender] == 0);
        balanceOf[msg.sender] = balanceOf[msg.sender].add(msg.value);
    }

    function withdraw() override public {
        require(msg.sender != lastSubmitter || block.timestamp >= lastValidAt);
        balanceOf[msg.sender] = balanceOf[msg.sender].sub(lockEthAmount);
        msg.sender.transfer(lockEthAmount);
    }

    function challenge(address payable receiver, uint256 signatureIndex) override public {
        require(block.timestamp < lastValidAt, "No block can be challenged at this time");

        require(
            !checkBlockProducerSignatureInHead(signatureIndex),
            "Can't challenge valid signature"
        );

        _payRewardAndRollBack(receiver);
    }

    function checkBlockProducerSignatureInHead(uint256 signatureIndex) override public view returns(bool) {
        BlockProducerInfo storage untrustedBlockProducers
            = untrustedHeadIsFromNextEpoch
            ? nextBlockProducers : currentBlockProducers;
        require(signatureIndex < untrustedBlockProducers.bpsLength, "Signature index out of range");
        require(!untrustedApprovals[signatureIndex].none, "This signature was skipped");
        return _checkValidatorSignature(
            untrustedHead.height,
            untrustedHead.next_hash,
            untrustedApprovals[signatureIndex].signature,
            untrustedBlockProducers.bps[signatureIndex].publicKey
        );
    }

    function _payRewardAndRollBack(address payable receiver) internal {
        // Pay reward
        balanceOf[lastSubmitter] = balanceOf[lastSubmitter].sub(lockEthAmount);
        receiver.transfer(lockEthAmount / 2);
        burner.transfer(lockEthAmount - lockEthAmount / 2);

        emit BlockHashReverted(
            untrustedHead.height,
            untrustedHead.hash
        );

        lastValidAt = 0;
    }

    // The first part of initialization -- setting the validators of the current epoch.
    function initWithValidators(bytes memory initialValidators_) override public {
        require(!initialized, "NearBridge: already initialized");

        Borsh.Data memory initialValidatorsBorsh = Borsh.from(initialValidators_);
        NearDecoder.InitialValidators memory initialValidators = initialValidatorsBorsh.decodeInitialValidators();
        require(initialValidatorsBorsh.finished(), "NearBridge: only initial validators should be passed as second argument");

        setBlockProducers(initialValidators.validator_stakes, currentBlockProducers);
    }

    // The second part of the initialization -- setting the current head.
    function initWithBlock(bytes memory data) override public {
        require(currentBlockProducers.totalStake > 0, "NearBridge: validators need to be initialized first");
        require(!initialized, "NearBridge: already initialized");
        initialized = true;

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh.decodeLightClientBlock();
        require(borsh.finished(), "NearBridge: only light client block should be passed as first argument");

        require(!nearBlock.next_bps.none, "NearBridge: Initialization block should contain next_bps.");
        setBlock(nearBlock, head);
        setBlockProducers(nearBlock.next_bps.validatorStakes, nextBlockProducers);
        blockHashes_[head.height] = head.hash;
        blockMerkleRoots_[head.height] = head.merkleRoot;
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

    struct BridgeState {
        uint currentHeight; // Height of the current confirmed block
        // If there is currently no unconfirmed block, the last three fields are zero.
        uint nextTimestamp; // Timestamp of the current unconfirmed block
        uint nextValidAt; // Timestamp when the current unconfirmed block will be confirmed
        uint numBlockProducers; // Number of block producers for the current unconfirmed block
    }

    function bridgeState() public view returns (BridgeState memory res) {
        if (block.timestamp < lastValidAt) {
            res.currentHeight = head.height;
            res.nextTimestamp = untrustedHead.timestamp;
            res.nextValidAt = lastValidAt;
            res.numBlockProducers =
                (untrustedHeadIsFromNextEpoch ? nextBlockProducers : currentBlockProducers)
                .bpsLength;
        } else {
            res.currentHeight = (lastValidAt == 0 ? head : untrustedHead).height;
        }
    }

    function addLightClientBlock(bytes memory data) override public {
        require(initialized, "NearBridge: Contract is not initialized.");
        require(balanceOf[msg.sender] >= lockEthAmount, "Balance is not enough");

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh.decodeLightClientBlock();
        require(borsh.finished(), "NearBridge: only light client block should be passed");

        // Commit the previous block, or make sure that it is OK to replace it.
        if (block.timestamp >= lastValidAt) {
            if (lastValidAt != 0) {
                commitBlock();
            }
        } else {
            require(nearBlock.inner_lite.timestamp >= untrustedHead.timestamp.add(replaceDuration), "NearBridge: can only replace with a sufficiently newer block");
        }

        // Check that the new block's height is greater than the current one's.
        require(
            nearBlock.inner_lite.height > head.height,
            "NearBridge: Height of the block is not valid"
        );

        // Check that the new block is from the same epoch as the current one, or from the next one.
        bool nearBlockIsFromNextEpoch;
        if (nearBlock.inner_lite.epoch_id == head.epochId) {
            nearBlockIsFromNextEpoch = false;
        } else if (nearBlock.inner_lite.epoch_id == head.nextEpochId) {
            nearBlockIsFromNextEpoch = true;
        } else {
            revert("NearBridge: Epoch id of the block is not valid");
        }

        // Check that the new block is signed by more than 2/3 of the validators.
        _checkBp(nearBlock, nearBlockIsFromNextEpoch ? nextBlockProducers : currentBlockProducers);

        // If the block is from the next epoch, make sure that next_bps is supplied and has a correct hash.
        if (nearBlockIsFromNextEpoch) {
            require(
                !nearBlock.next_bps.none,
                "NearBridge: Next next_bps should not be None"
            );
            require(
                nearBlock.next_bps.hash == nearBlock.inner_lite.next_bp_hash,
                "NearBridge: Hash of block producers does not match"
            );
        }

        setBlock(nearBlock, untrustedHead);
        untrustedApprovalCount = nearBlock.approvals_after_next.length;
        for (uint i = 0; i < nearBlock.approvals_after_next.length; i++) {
            untrustedApprovals[i] = nearBlock.approvals_after_next[i];
        }
        untrustedHeadIsFromNextEpoch = nearBlockIsFromNextEpoch;
        if (nearBlockIsFromNextEpoch) {
            setBlockProducers(nearBlock.next_bps.validatorStakes, untrustedNextBlockProducers);
        }
        lastSubmitter = msg.sender;
        lastValidAt = block.timestamp.add(lockDuration);
    }

    function setBlock(NearDecoder.LightClientBlock memory src, BlockInfo storage dest) internal {
        dest.height = src.inner_lite.height;
        dest.timestamp = src.inner_lite.timestamp;
        dest.epochId = src.inner_lite.epoch_id;
        dest.nextEpochId = src.inner_lite.next_epoch_id;
        dest.hash = src.hash;
        dest.merkleRoot = src.inner_lite.block_merkle_root;
        dest.next_hash = src.next_hash;

        emit BlockHashAdded(
            src.inner_lite.height,
            src.hash
        );
    }

    function setBlockProducers(NearDecoder.ValidatorStake[] memory src, BlockProducerInfo storage dest) internal {
        dest.bpsLength = src.length;
        uint256 totalStake = 0;
        for (uint i = 0; i < src.length; i++) {
            dest.bps[i] = BlockProducer({
                publicKey: src[i].public_key,
                stake: src[i].stake
            });
            totalStake = totalStake.add(src[i].stake);
        }
        dest.totalStake = totalStake;
    }


    function commitBlock() internal {
        require(lastValidAt != 0 && block.timestamp >= lastValidAt, "Nothing to commit");

        head = untrustedHead;
        if (untrustedHeadIsFromNextEpoch) {
            // Switch to the next epoch. It is guaranteed that untrustedNextBlockProducers is set.
            copyBlockProducers(nextBlockProducers, currentBlockProducers);
            copyBlockProducers(untrustedNextBlockProducers, nextBlockProducers);
        }
        lastValidAt = 0;

        blockHashes_[head.height] = head.hash;
        blockMerkleRoots_[head.height] = head.merkleRoot;
    }

    function copyBlockProducers(BlockProducerInfo storage src, BlockProducerInfo storage dest) internal {
        dest.bpsLength = src.bpsLength;
        dest.totalStake = src.totalStake;
        for (uint i = 0; i < src.bpsLength; i++) {
            dest.bps[i] = src.bps[i];
        }
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

    function blockHashes(uint64 height) override public view returns (bytes32 res) {
        res = blockHashes_[height];
        if (res == 0 && block.timestamp >= lastValidAt && lastValidAt != 0 && height == untrustedHead.height) {
            res = untrustedHead.hash;
        }
    }

    function blockMerkleRoots(uint64 height) override public view returns (bytes32 res) {
        res = blockMerkleRoots_[height];
        if (res == 0 && block.timestamp >= lastValidAt && lastValidAt != 0 && height == untrustedHead.height) {
            res = untrustedHead.merkleRoot;
        }
    }
}
