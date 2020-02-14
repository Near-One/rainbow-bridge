pragma solidity ^0.5.0;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "./Borsh.sol";


library NearDecoder {

    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;

    struct BlockHeaderInnerLite {
        uint64 height;              /// Height of this block since the genesis block (height 0).
        byte[32] epoch_id;          /// Epoch start hash of this block's epoch. Used for retrieving validator information
        byte[32] next_epoch_id;
        byte[32] prev_state_root;   /// Root hash of the state at the previous block.
        byte[32] outcome_root;      /// Root of the outcomes of transactions and receipts.
        uint64 timestamp;           /// Timestamp at which the block was built.
        byte[32] next_bp_hash;      /// Hash of the next epoch block producers set
    }

    function decodeBlockHeaderInnerLite(Borsh.Data memory data) internal pure returns(BlockHeaderInnerLite memory header) {
        header.height = data.decodeU64();
        header.epoch_id = data.decodeBytes32();
        header.next_epoch_id = data.decodeBytes32();
        header.prev_state_root = data.decodeBytes32();
        header.outcome_root = data.decodeBytes32();
        header.timestamp = data.decodeU64();
        header.next_bp_hash = data.decodeBytes32();
    }

    struct ExecutionStatus {
        uint8 enumIndex;
        bool unknown;
        bool failed;
        bytes successValue;         /// The final action succeeded and returned some value or an empty vec.
        byte[32] successReceiptId;  /// The final action of the receipt returned a promise or the signed
                                    /// transaction was converted to a receipt. Contains the receipt_id of the generated receipt.
    }

    function decodeExecutionStatus(Borsh.Data memory data) internal pure returns(ExecutionStatus memory executionStatus) {
        executionStatus.enumIndex = data.decodeU8();
        if (executionStatus.enumIndex == 0) {
            executionStatus.unknown = true;
        } else
        if (executionStatus.enumIndex == 1) {
            //revert("NearDecoder: decodeExecutionStatus failure case not implemented yet");
            // Can avoid revert since ExecutionStatus is latest field in all parent structures
            executionStatus.failed = true;
        } else
        if (executionStatus.enumIndex == 2) {
            executionStatus.successValue = data.decodeBytes();
        } else
        if (executionStatus.enumIndex == 3) {
            executionStatus.successReceiptId = data.decodeBytes32();
        } else {
            revert("NearDecoder: decodeExecutionStatus index out of range");
        }
    }

    struct ExecutionOutcome {
        bytes[] logs;           /// Logs from this transaction or receipt.
        byte[32][] receipt_ids; /// Receipt IDs generated by this transaction or receipt.
        uint64 gas_burnt;       /// The amount of the gas burnt by the given transaction or receipt.
        ExecutionStatus status; /// Execution status. Contains the result in case of successful execution.
    }

    function decodeExecutionOutcome(Borsh.Data memory data) internal pure returns(ExecutionOutcome memory outcome) {
        outcome.logs = new bytes[](data.decodeU32());
        for (uint i = 0; i < outcome.logs.length; i++) {
            outcome.logs[i] = data.decodeBytes();
        }
        outcome.receipt_ids = new byte[32][](data.decodeU32());
        for (uint i = 0; i < outcome.receipt_ids.length; i++) {
            outcome.receipt_ids[i] = data.decodeBytes32();
        }
        outcome.gas_burnt = data.decodeU64();
        outcome.status = data.decodeExecutionStatus();
    }

    struct ExecutionOutcomeWithId {
        byte[32] id;                /// The transaction hash or the receipt ID.
        ExecutionOutcome outcome;
    }

    function decodeExecutionOutcomeWithId(Borsh.Data memory data) internal pure returns(ExecutionOutcomeWithId memory outcome) {
        outcome.id = data.decodeBytes32();
        outcome.outcome = data.decodeExecutionOutcome();
    }

    struct MerklePathItem {
        byte[32] hash;
        uint8 direction; // 0 = left, 1 = right
    }

    function decodeMerklePathItem(Borsh.Data memory data) internal pure returns(MerklePathItem memory item) {
        item.hash = data.decodeBytes32();
        item.direction = data.decodeU8();
    }

    struct MerklePath {
        MerklePathItem[] items;
    }

    function decodeMerklePath(Borsh.Data memory data) internal pure returns(MerklePath memory path) {
        path.items = new MerklePathItem[](data.decodeU32());
        for (uint i = 0; i < path.items.length; i++) {
            path.items[i] = data.decodeMerklePathItem();
        }
    }

    struct ExecutionOutcomeWithIdAndProof {
        MerklePath proof;
        byte[32] block_hash;
        ExecutionOutcomeWithId outcome_with_id;
    }

    function decodeExecutionOutcomeWithIdAndProof(Borsh.Data memory data)
        internal
        pure
        returns(ExecutionOutcomeWithIdAndProof memory outcome)
    {
        outcome.proof = data.decodeMerklePath();
        outcome.block_hash = data.decodeBytes32();
        outcome.outcome_with_id = data.decodeExecutionOutcomeWithId();
    }
}
