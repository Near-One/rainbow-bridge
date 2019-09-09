pragma solidity ^0.5.0;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./Borsh.sol";


library NearDecoder {

    using Borsh for Borsh.Data;

    struct PublicKey {
        uint8 enumIndex;
        byte[32] ed25519;
        byte[64] secp256k1;
    }

    struct Signature {
        uint8 enumIndex;
        byte[64] ed25519;
        byte[65] secp256k1;
    }

    struct ValidatorStake {
        string account_id;    /// Account that stakes money.
        PublicKey public_key; /// Public key of the proposed validator.
        uint128 amount;       /// Stake / weight of the validator.
    }

    struct BlockHeaderInner {
        uint64 height;                        /// Height of this block since the genesis block (height 0).
        byte[32] epoch_id;                    /// Epoch start hash of this block's epoch. Used for retrieving validator information
        byte[32] prev_hash;                   /// Hash of the block previous to this in the chain.
        byte[32] prev_state_root;             /// Root hash of the state at the previous block.
        byte[32] tx_root;                     /// Root hash of the transactions in the given block.
        uint64 timestamp;                     /// Timestamp at which the block was built.
        bool[] approval_mask;                 /// Approval mask, given current block producers.
        Signature[] approval_sigs;            /// Approval signatures for previous block.
        uint64 total_weight;                  /// Total weight.
        ValidatorStake[] validator_proposals; /// Validator proposals.
        bool[] chunk_mask;                    /// Mask for new chunks included in the block
        uint64 gas_used;                      /// Sum of gas used across all chunks.
        uint64 gas_limit;                     /// Gas limit. Same for all chunks.
        uint128 gas_price;                    /// Gas price. Same for all chunks
        uint128 total_supply;
    }

    struct BlockHeader {
        BlockHeaderInner inner; /// Inner part of the block header that gets hashed.
        Signature signature;    /// Signature of the block producer.
    }

    // struct SignedTransaction {
    //     Transaction transaction;
    //     Signature signature;
    //     byte[32] hash;
    // }

    // struct Block {
    //     BlockHeader header;
    //     SignedTransaction[] transactions;
    // }

    enum TransactionStatus {
        Unknown,
        Completed,
        Failed
    }

    struct TransactionResult {
        TransactionStatus status; /// Transaction status.
        string[] logs;            /// Logs from this transaction.
        byte[32][] receipts;      /// Receipt ids generated by this transaction.
        bytes result; //?         /// Execution Result (Optional)
    }

    struct TransactionLog {
        byte[32] hash;            /// Hash of a transaction or a receipt that generated this result.
        TransactionResult result;
    }

    function decodePublicKey(Borsh.Data memory data) private pure returns(PublicKey memory publicKey) {
        publicKey.enumIndex = data.decodeU8();
        if (publicKey.enumIndex == 0) {
            data.decodeBytes32To(publicKey.ed25519);
        } else
        if (publicKey.enumIndex == 1) {
            data.decodeBytes64To(publicKey.secp256k1);
        }
    }

    function decodeSignature(Borsh.Data memory data) private pure returns(Signature memory signature) {
        signature.enumIndex = data.decodeU8();
        if (signature.enumIndex == 0) {
            data.decodeBytes64To(signature.ed25519);
        } else
        if (signature.enumIndex == 1) {
            data.decodeBytes65To(signature.secp256k1);
        }
    }

    function decodeValidatorStake(Borsh.Data memory data) private pure returns(ValidatorStake memory validatorStake) {
        validatorStake.account_id = string(data.decodeBytes());
        validatorStake.public_key = decodePublicKey(data);
        validatorStake.amount = data.decodeU128();
    }

    function decodeBlockHeaderInner(Borsh.Data memory data) private pure returns(BlockHeaderInner memory header) {
        header.height = data.decodeU64();
        data.decodeBytes32To(header.epoch_id);
        data.decodeBytes32To(header.prev_hash);
        data.decodeBytes32To(header.prev_state_root);
        data.decodeBytes32To(header.tx_root);
        header.timestamp = data.decodeU64();

        uint256 approval_mask_length = data.decodeU32();
        header.approval_mask = new bool[](approval_mask_length);
        for (uint i = 0; i < approval_mask_length; i++) {
            header.approval_mask[i] = data.decodeBool();
        }

        uint256 approval_sigs_length = data.decodeU32();
        header.approval_sigs = new Signature[](approval_sigs_length);
        for (uint i = 0; i < approval_sigs_length; i++) {
            header.approval_sigs[i] = decodeSignature(data);
        }

        header.total_weight = data.decodeU64();

        uint256 validator_proposals_length = data.decodeU32();
        header.validator_proposals = new ValidatorStake[](validator_proposals_length);
        for (uint i = 0; i < validator_proposals_length; i++) {
            header.validator_proposals[i] = decodeValidatorStake(data);
        }

        uint256 chunk_mask_length = data.decodeU32();
        header.chunk_mask = new bool[](chunk_mask_length);
        for (uint i = 0; i < chunk_mask_length; i++) {
            header.chunk_mask[i] = data.decodeBool();
        }

        header.gas_used = data.decodeU64();
        header.gas_limit = data.decodeU64();
        header.gas_price = data.decodeU128();
        header.total_supply = data.decodeU128();
    }

    function decodeBlockHeader(Borsh.Data memory data) private pure returns(BlockHeader memory header) {
        header.inner = decodeBlockHeaderInner(data);
        header.signature = decodeSignature(data);
    }

    function decodeTransactionResult(Borsh.Data memory data) private pure returns(TransactionResult memory result) {
        result.status = TransactionStatus(data.decodeU8());

        uint256 logs_length = data.decodeU32();
        result.logs = new string[](logs_length);
        for (uint i = 0; i < logs_length; i++) {
            result.logs[i] = string(data.decodeBytes());
        }

        uint256 receipts_length = data.decodeU32();
        result.receipts = new byte[32][](receipts_length);
        for (uint i = 0; i < receipts_length; i++) {
            data.decodeBytes32To(result.receipts[i]);
        }

        bool resultPresented = data.decodeBool();
        if (resultPresented) {
            result.result = data.decodeBytes();
        }
    }

    function decodeTransactionLog(Borsh.Data memory data) private pure returns(TransactionLog memory log) {
        data.decodeBytes32To(log.hash);
        log.result = decodeTransactionResult(data);
    }
}
