// File: @openzeppelin/contracts/math/SafeMath.sol


pragma solidity >=0.6.0 <0.8.0;

/**
 * @dev Wrappers over Solidity's arithmetic operations with added overflow
 * checks.
 *
 * Arithmetic operations in Solidity wrap on overflow. This can easily result
 * in bugs, because programmers usually assume that an overflow raises an
 * error, which is the standard behavior in high level programming languages.
 * `SafeMath` restores this intuition by reverting the transaction when an
 * operation overflows.
 *
 * Using this library instead of the unchecked operations eliminates an entire
 * class of bugs, so it's recommended to use it always.
 */
library SafeMath {
    /**
     * @dev Returns the addition of two unsigned integers, with an overflow flag.
     *
     * _Available since v3.4._
     */
    function tryAdd(uint256 a, uint256 b) internal pure returns (bool, uint256) {
        uint256 c = a + b;
        if (c < a) return (false, 0);
        return (true, c);
    }

    /**
     * @dev Returns the substraction of two unsigned integers, with an overflow flag.
     *
     * _Available since v3.4._
     */
    function trySub(uint256 a, uint256 b) internal pure returns (bool, uint256) {
        if (b > a) return (false, 0);
        return (true, a - b);
    }

    /**
     * @dev Returns the multiplication of two unsigned integers, with an overflow flag.
     *
     * _Available since v3.4._
     */
    function tryMul(uint256 a, uint256 b) internal pure returns (bool, uint256) {
        // Gas optimization: this is cheaper than requiring 'a' not being zero, but the
        // benefit is lost if 'b' is also tested.
        // See: https://github.com/OpenZeppelin/openzeppelin-contracts/pull/522
        if (a == 0) return (true, 0);
        uint256 c = a * b;
        if (c / a != b) return (false, 0);
        return (true, c);
    }

    /**
     * @dev Returns the division of two unsigned integers, with a division by zero flag.
     *
     * _Available since v3.4._
     */
    function tryDiv(uint256 a, uint256 b) internal pure returns (bool, uint256) {
        if (b == 0) return (false, 0);
        return (true, a / b);
    }

    /**
     * @dev Returns the remainder of dividing two unsigned integers, with a division by zero flag.
     *
     * _Available since v3.4._
     */
    function tryMod(uint256 a, uint256 b) internal pure returns (bool, uint256) {
        if (b == 0) return (false, 0);
        return (true, a % b);
    }

    /**
     * @dev Returns the addition of two unsigned integers, reverting on
     * overflow.
     *
     * Counterpart to Solidity's `+` operator.
     *
     * Requirements:
     *
     * - Addition cannot overflow.
     */
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        uint256 c = a + b;
        require(c >= a, "SafeMath: addition overflow");
        return c;
    }

    /**
     * @dev Returns the subtraction of two unsigned integers, reverting on
     * overflow (when the result is negative).
     *
     * Counterpart to Solidity's `-` operator.
     *
     * Requirements:
     *
     * - Subtraction cannot overflow.
     */
    function sub(uint256 a, uint256 b) internal pure returns (uint256) {
        require(b <= a, "SafeMath: subtraction overflow");
        return a - b;
    }

    /**
     * @dev Returns the multiplication of two unsigned integers, reverting on
     * overflow.
     *
     * Counterpart to Solidity's `*` operator.
     *
     * Requirements:
     *
     * - Multiplication cannot overflow.
     */
    function mul(uint256 a, uint256 b) internal pure returns (uint256) {
        if (a == 0) return 0;
        uint256 c = a * b;
        require(c / a == b, "SafeMath: multiplication overflow");
        return c;
    }

    /**
     * @dev Returns the integer division of two unsigned integers, reverting on
     * division by zero. The result is rounded towards zero.
     *
     * Counterpart to Solidity's `/` operator. Note: this function uses a
     * `revert` opcode (which leaves remaining gas untouched) while Solidity
     * uses an invalid opcode to revert (consuming all remaining gas).
     *
     * Requirements:
     *
     * - The divisor cannot be zero.
     */
    function div(uint256 a, uint256 b) internal pure returns (uint256) {
        require(b > 0, "SafeMath: division by zero");
        return a / b;
    }

    /**
     * @dev Returns the remainder of dividing two unsigned integers. (unsigned integer modulo),
     * reverting when dividing by zero.
     *
     * Counterpart to Solidity's `%` operator. This function uses a `revert`
     * opcode (which leaves remaining gas untouched) while Solidity uses an
     * invalid opcode to revert (consuming all remaining gas).
     *
     * Requirements:
     *
     * - The divisor cannot be zero.
     */
    function mod(uint256 a, uint256 b) internal pure returns (uint256) {
        require(b > 0, "SafeMath: modulo by zero");
        return a % b;
    }

    /**
     * @dev Returns the subtraction of two unsigned integers, reverting with custom message on
     * overflow (when the result is negative).
     *
     * CAUTION: This function is deprecated because it requires allocating memory for the error
     * message unnecessarily. For custom revert reasons use {trySub}.
     *
     * Counterpart to Solidity's `-` operator.
     *
     * Requirements:
     *
     * - Subtraction cannot overflow.
     */
    function sub(uint256 a, uint256 b, string memory errorMessage) internal pure returns (uint256) {
        require(b <= a, errorMessage);
        return a - b;
    }

    /**
     * @dev Returns the integer division of two unsigned integers, reverting with custom message on
     * division by zero. The result is rounded towards zero.
     *
     * CAUTION: This function is deprecated because it requires allocating memory for the error
     * message unnecessarily. For custom revert reasons use {tryDiv}.
     *
     * Counterpart to Solidity's `/` operator. Note: this function uses a
     * `revert` opcode (which leaves remaining gas untouched) while Solidity
     * uses an invalid opcode to revert (consuming all remaining gas).
     *
     * Requirements:
     *
     * - The divisor cannot be zero.
     */
    function div(uint256 a, uint256 b, string memory errorMessage) internal pure returns (uint256) {
        require(b > 0, errorMessage);
        return a / b;
    }

    /**
     * @dev Returns the remainder of dividing two unsigned integers. (unsigned integer modulo),
     * reverting with custom message when dividing by zero.
     *
     * CAUTION: This function is deprecated because it requires allocating memory for the error
     * message unnecessarily. For custom revert reasons use {tryMod}.
     *
     * Counterpart to Solidity's `%` operator. This function uses a `revert`
     * opcode (which leaves remaining gas untouched) while Solidity uses an
     * invalid opcode to revert (consuming all remaining gas).
     *
     * Requirements:
     *
     * - The divisor cannot be zero.
     */
    function mod(uint256 a, uint256 b, string memory errorMessage) internal pure returns (uint256) {
        require(b > 0, errorMessage);
        return a % b;
    }
}

// File: contracts/bridge/AdminControlled.sol

pragma solidity ^0.6;

contract AdminControlled {
    address public admin;
    uint public paused;

    constructor(address _admin, uint flags) public {
        admin = _admin;
        paused = flags;
    }

    modifier onlyAdmin {
        require(msg.sender == admin);
        _;
    }

    modifier pausable(uint flag) {
        require((paused & flag) == 0 || msg.sender == admin);
        _;
    }

    function adminPause(uint flags) public onlyAdmin {
        paused = flags;
    }

    function adminSstore(uint key, uint value) public onlyAdmin {
        assembly {
            sstore(key, value)
        }
    }

    function adminSstoreWithMask(
        uint key,
        uint value,
        uint mask
    ) public onlyAdmin {
        assembly {
            let oldval := sload(key)
            sstore(key, xor(and(xor(value, oldval), mask), oldval))
        }
    }

    function adminSendEth(address payable destination, uint amount) public onlyAdmin {
        destination.transfer(amount);
    }

    function adminReceiveEth() public payable onlyAdmin {}

    function adminDelegatecall(address target, bytes memory data) public payable onlyAdmin returns (bytes memory) {
        (bool success, bytes memory rdata) = target.delegatecall(data);
        require(success);
        return rdata;
    }
}

// File: contracts/bridge/INearBridge.sol

pragma solidity ^0.6;

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

// File: contracts/bridge/Utils.sol

pragma solidity ^0.6;

library Utils {
    function swapBytes2(uint16 v) internal pure returns (uint16) {
        return (v << 8) | (v >> 8);
    }

    function swapBytes4(uint32 v) internal pure returns (uint32) {
        v = ((v & 0x00ff00ff) << 8) | ((v & 0xff00ff00) >> 8);
        return (v << 16) | (v >> 16);
    }

    function swapBytes8(uint64 v) internal pure returns (uint64) {
        v = ((v & 0x00ff00ff00ff00ff) << 8) | ((v & 0xff00ff00ff00ff00) >> 8);
        v = ((v & 0x0000ffff0000ffff) << 16) | ((v & 0xffff0000ffff0000) >> 16);
        return (v << 32) | (v >> 32);
    }

    function swapBytes16(uint128 v) internal pure returns (uint128) {
        v = ((v & 0x00ff00ff00ff00ff00ff00ff00ff00ff) << 8) | ((v & 0xff00ff00ff00ff00ff00ff00ff00ff00) >> 8);
        v = ((v & 0x0000ffff0000ffff0000ffff0000ffff) << 16) | ((v & 0xffff0000ffff0000ffff0000ffff0000) >> 16);
        v = ((v & 0x00000000ffffffff00000000ffffffff) << 32) | ((v & 0xffffffff00000000ffffffff00000000) >> 32);
        return (v << 64) | (v >> 64);
    }

    function swapBytes32(uint256 v) internal pure returns (uint256) {
        v =
            ((v & 0x00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff) << 8) |
            ((v & 0xff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00) >> 8);
        v =
            ((v & 0x0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff) << 16) |
            ((v & 0xffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000ffff0000) >> 16);
        v =
            ((v & 0x00000000ffffffff00000000ffffffff00000000ffffffff00000000ffffffff) << 32) |
            ((v & 0xffffffff00000000ffffffff00000000ffffffff00000000ffffffff00000000) >> 32);
        v =
            ((v & 0x0000000000000000ffffffffffffffff0000000000000000ffffffffffffffff) << 64) |
            ((v & 0xffffffffffffffff0000000000000000ffffffffffffffff0000000000000000) >> 64);
        return (v << 128) | (v >> 128);
    }

    function readMemory(uint ptr) internal pure returns (uint res) {
        assembly {
            res := mload(ptr)
        }
    }

    function writeMemory(uint ptr, uint value) internal pure {
        assembly {
            mstore(ptr, value)
        }
    }

    function memoryToBytes(uint ptr, uint length) internal pure returns (bytes memory res) {
        if (length != 0) {
            assembly {
                // 0x40 is the address of free memory pointer.
                res := mload(0x40)
                let end := add(
                    res,
                    and(add(length, 63), 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0)
                )
                // end = res + 32 + 32 * ceil(length / 32).
                mstore(0x40, end)
                mstore(res, length)
                let destPtr := add(res, 32)
                // prettier-ignore
                for { } 1 { } {
                    mstore(destPtr, mload(ptr))
                    destPtr := add(destPtr, 32)
                    if eq(destPtr, end) {
                        break
                    }
                    ptr := add(ptr, 32)
                }
            }
        }
    }

    function keccak256Raw(uint ptr, uint length) internal pure returns (bytes32 res) {
        assembly {
            res := keccak256(ptr, length)
        }
    }

    function sha256Raw(uint ptr, uint length) internal view returns (bytes32 res) {
        assembly {
            // 2 is the address of SHA256 precompiled contract.
            // First 64 bytes of memory can be used as scratch space.
            let ret := staticcall(gas(), 2, ptr, length, 0, 32)
            // If the call to SHA256 precompile ran out of gas, burn any gas that remains.
            // prettier-ignore
            for { } iszero(ret) { } { }
            res := mload(0)
        }
    }
}

// File: contracts/bridge/Borsh.sol

pragma solidity ^0.6;


library Borsh {
    using Borsh for Data;

    struct Data {
        uint ptr;
        uint end;
    }

    function from(bytes memory data) internal pure returns (Data memory res) {
        uint ptr;
        assembly {
            ptr := data
        }
        res.ptr = ptr + 32;
        res.end = res.ptr + Utils.readMemory(ptr);
    }

    // This function assumes that length is reasonably small, so that data.ptr + length will not overflow. In the current code, length is always less than 2^32.
    function requireSpace(Data memory data, uint length) internal pure {
        require(data.ptr + length <= data.end, "Parse error: unexpected EOI");
    }

    function read(Data memory data, uint length) internal pure returns (bytes32 res) {
        data.requireSpace(length);
        res = bytes32(Utils.readMemory(data.ptr));
        data.ptr += length;
        return res;
    }

    function done(Data memory data) internal pure {
        require(data.ptr == data.end, "Parse error: EOI expected");
    }

    // Same considerations as for requireSpace.
    function peekKeccak256(Data memory data, uint length) internal pure returns (bytes32) {
        data.requireSpace(length);
        return Utils.keccak256Raw(data.ptr, length);
    }

    // Same considerations as for requireSpace.
    function peekSha256(Data memory data, uint length) internal view returns (bytes32) {
        data.requireSpace(length);
        return Utils.sha256Raw(data.ptr, length);
    }

    function decodeU8(Data memory data) internal pure returns (uint8) {
        return uint8(bytes1(data.read(1)));
    }

    function decodeU16(Data memory data) internal pure returns (uint16) {
        return Utils.swapBytes2(uint16(bytes2(data.read(2))));
    }

    function decodeU32(Data memory data) internal pure returns (uint32) {
        return Utils.swapBytes4(uint32(bytes4(data.read(4))));
    }

    function decodeU64(Data memory data) internal pure returns (uint64) {
        return Utils.swapBytes8(uint64(bytes8(data.read(8))));
    }

    function decodeU128(Data memory data) internal pure returns (uint128) {
        return Utils.swapBytes16(uint128(bytes16(data.read(16))));
    }

    function decodeU256(Data memory data) internal pure returns (uint256) {
        return Utils.swapBytes32(uint256(data.read(32)));
    }

    function decodeBytes20(Data memory data) internal pure returns (bytes20) {
        return bytes20(data.read(20));
    }

    function decodeBytes32(Data memory data) internal pure returns (bytes32) {
        return data.read(32);
    }

    function decodeBool(Data memory data) internal pure returns (bool) {
        uint8 res = data.decodeU8();
        require(res <= 1, "Parse error: invalid bool");
        return res != 0;
    }

    function skipBytes(Data memory data) internal pure {
        uint length = data.decodeU32();
        data.requireSpace(length);
        data.ptr += length;
    }

    function decodeBytes(Data memory data) internal pure returns (bytes memory res) {
        uint length = data.decodeU32();
        data.requireSpace(length);
        res = Utils.memoryToBytes(data.ptr, length);
        data.ptr += length;
    }
}

// File: contracts/bridge/NearDecoder.sol

pragma solidity ^0.6;


library NearDecoder {
    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;

    struct PublicKey {
        bytes32 k;
    }

    function decodePublicKey(Borsh.Data memory data) internal pure returns (PublicKey memory res) {
        require(data.decodeU8() == 0, "Parse error: invalid key type");
        res.k = data.decodeBytes32();
    }

    struct Signature {
        bytes32 r;
        bytes32 s;
    }

    function decodeSignature(Borsh.Data memory data) internal pure returns (Signature memory res) {
        require(data.decodeU8() == 0, "Parse error: invalid signature type");
        res.r = data.decodeBytes32();
        res.s = data.decodeBytes32();
    }

    struct BlockProducer {
        PublicKey publicKey;
        uint128 stake;
    }

    function decodeBlockProducer(Borsh.Data memory data) internal pure returns (BlockProducer memory res) {
        data.skipBytes();
        res.publicKey = data.decodePublicKey();
        res.stake = data.decodeU128();
    }

    function decodeBlockProducers(Borsh.Data memory data) internal pure returns (BlockProducer[] memory res) {
        uint length = data.decodeU32();
        res = new BlockProducer[](length);
        for (uint i = 0; i < length; i++) {
            res[i] = data.decodeBlockProducer();
        }
    }

    struct OptionalBlockProducers {
        bool some;
        BlockProducer[] blockProducers;
        bytes32 hash; // Additional computable element
    }

    function decodeOptionalBlockProducers(Borsh.Data memory data)
        internal
        view
        returns (OptionalBlockProducers memory res)
    {
        res.some = data.decodeBool();
        if (res.some) {
            uint start = data.ptr;
            res.blockProducers = data.decodeBlockProducers();
            res.hash = Utils.sha256Raw(start, data.ptr - start);
        }
    }

    struct OptionalSignature {
        bool some;
        Signature signature;
    }

    function decodeOptionalSignature(Borsh.Data memory data) internal pure returns (OptionalSignature memory res) {
        res.some = data.decodeBool();
        if (res.some) {
            res.signature = data.decodeSignature();
        }
    }

    struct BlockHeaderInnerLite {
        uint64 height; // Height of this block since the genesis block (height 0).
        bytes32 epoch_id; // Epoch start hash of this block's epoch. Used for retrieving validator information
        bytes32 next_epoch_id;
        bytes32 prev_state_root; // Root hash of the state at the previous block.
        bytes32 outcome_root; // Root of the outcomes of transactions and receipts.
        uint64 timestamp; // Timestamp at which the block was built.
        bytes32 next_bp_hash; // Hash of the next epoch block producers set
        bytes32 block_merkle_root;
        bytes32 hash; // Additional computable element
    }

    function decodeBlockHeaderInnerLite(Borsh.Data memory data)
        internal
        view
        returns (BlockHeaderInnerLite memory res)
    {
        res.hash = data.peekSha256(208);
        res.height = data.decodeU64();
        res.epoch_id = data.decodeBytes32();
        res.next_epoch_id = data.decodeBytes32();
        res.prev_state_root = data.decodeBytes32();
        res.outcome_root = data.decodeBytes32();
        res.timestamp = data.decodeU64();
        res.next_bp_hash = data.decodeBytes32();
        res.block_merkle_root = data.decodeBytes32();
    }

    struct LightClientBlock {
        bytes32 prev_block_hash;
        bytes32 next_block_inner_hash;
        BlockHeaderInnerLite inner_lite;
        bytes32 inner_rest_hash;
        OptionalBlockProducers next_bps;
        OptionalSignature[] approvals_after_next;
        bytes32 hash;
        bytes32 next_hash;
    }

    function decodeLightClientBlock(Borsh.Data memory data) internal view returns (LightClientBlock memory res) {
        res.prev_block_hash = data.decodeBytes32();
        res.next_block_inner_hash = data.decodeBytes32();
        res.inner_lite = data.decodeBlockHeaderInnerLite();
        res.inner_rest_hash = data.decodeBytes32();
        res.next_bps = data.decodeOptionalBlockProducers();

        uint length = data.decodeU32();
        res.approvals_after_next = new OptionalSignature[](length);
        for (uint i = 0; i < length; i++) {
            res.approvals_after_next[i] = data.decodeOptionalSignature();
        }

        res.hash = sha256(
            abi.encodePacked(sha256(abi.encodePacked(res.inner_lite.hash, res.inner_rest_hash)), res.prev_block_hash)
        );

        res.next_hash = sha256(abi.encodePacked(res.next_block_inner_hash, res.hash));
    }
}

// File: contracts/ProofDecoder.sol

pragma solidity ^0.6;



library ProofDecoder {
    using Borsh for Borsh.Data;
    using ProofDecoder for Borsh.Data;
    using NearDecoder for Borsh.Data;

    struct FullOutcomeProof {
        ExecutionOutcomeWithIdAndProof outcome_proof;
        MerklePath outcome_root_proof; // TODO: now empty array
        BlockHeaderLight block_header_lite;
        MerklePath block_proof;
    }

    function decodeFullOutcomeProof(Borsh.Data memory data) internal view returns (FullOutcomeProof memory proof) {
        proof.outcome_proof = data.decodeExecutionOutcomeWithIdAndProof();
        proof.outcome_root_proof = data.decodeMerklePath();
        proof.block_header_lite = data.decodeBlockHeaderLight();
        proof.block_proof = data.decodeMerklePath();
    }

    struct BlockHeaderLight {
        bytes32 prev_block_hash;
        bytes32 inner_rest_hash;
        NearDecoder.BlockHeaderInnerLite inner_lite;
        bytes32 hash; // Computable
    }

    function decodeBlockHeaderLight(Borsh.Data memory data) internal view returns (BlockHeaderLight memory header) {
        header.prev_block_hash = data.decodeBytes32();
        header.inner_rest_hash = data.decodeBytes32();
        header.inner_lite = data.decodeBlockHeaderInnerLite();

        header.hash = sha256(
            abi.encodePacked(
                sha256(abi.encodePacked(header.inner_lite.hash, header.inner_rest_hash)),
                header.prev_block_hash
            )
        );
    }

    struct ExecutionStatus {
        uint8 enumIndex;
        bool unknown;
        bool failed;
        bytes successValue; // The final action succeeded and returned some value or an empty vec.
        bytes32 successReceiptId; // The final action of the receipt returned a promise or the signed transaction was converted to a receipt. Contains the receipt_id of the generated receipt.
    }

    function decodeExecutionStatus(Borsh.Data memory data)
        internal
        pure
        returns (ExecutionStatus memory executionStatus)
    {
        executionStatus.enumIndex = data.decodeU8();
        if (executionStatus.enumIndex == 0) {
            executionStatus.unknown = true;
        } else if (executionStatus.enumIndex == 1) {
            //revert("NearDecoder: decodeExecutionStatus failure case not implemented yet");
            // Can avoid revert since ExecutionStatus is latest field in all parent structures
            executionStatus.failed = true;
        } else if (executionStatus.enumIndex == 2) {
            executionStatus.successValue = data.decodeBytes();
        } else if (executionStatus.enumIndex == 3) {
            executionStatus.successReceiptId = data.decodeBytes32();
        } else {
            revert("NearDecoder: decodeExecutionStatus index out of range");
        }
    }

    struct ExecutionOutcome {
        bytes[] logs; // Logs from this transaction or receipt.
        bytes32[] receipt_ids; // Receipt IDs generated by this transaction or receipt.
        uint64 gas_burnt; // The amount of the gas burnt by the given transaction or receipt.
        uint128 tokens_burnt; // The total number of the tokens burnt by the given transaction or receipt.
        bytes executor_id; // Hash of the transaction or receipt id that produced this outcome.
        ExecutionStatus status; // Execution status. Contains the result in case of successful execution.
        bytes32[] merkelization_hashes;
    }

    function decodeExecutionOutcome(Borsh.Data memory data) internal view returns (ExecutionOutcome memory outcome) {
        outcome.logs = new bytes[](data.decodeU32());
        for (uint i = 0; i < outcome.logs.length; i++) {
            outcome.logs[i] = data.decodeBytes();
        }

        uint256 start = data.ptr;
        outcome.receipt_ids = new bytes32[](data.decodeU32());
        for (uint i = 0; i < outcome.receipt_ids.length; i++) {
            outcome.receipt_ids[i] = data.decodeBytes32();
        }
        outcome.gas_burnt = data.decodeU64();
        outcome.tokens_burnt = data.decodeU128();
        outcome.executor_id = data.decodeBytes();
        outcome.status = data.decodeExecutionStatus();

        outcome.merkelization_hashes = new bytes32[](1 + outcome.logs.length);
        outcome.merkelization_hashes[0] = Utils.sha256Raw(start, data.ptr - start);
        for (uint i = 0; i < outcome.logs.length; i++) {
            outcome.merkelization_hashes[i + 1] = sha256(outcome.logs[i]);
        }
    }

    struct ExecutionOutcomeWithId {
        bytes32 id; // The transaction hash or the receipt ID.
        ExecutionOutcome outcome;
        bytes32 hash;
    }

    function decodeExecutionOutcomeWithId(Borsh.Data memory data)
        internal
        view
        returns (ExecutionOutcomeWithId memory outcome)
    {
        outcome.id = data.decodeBytes32();
        outcome.outcome = data.decodeExecutionOutcome();

        uint256 len = 1 + outcome.outcome.merkelization_hashes.length;
        outcome.hash = sha256(
            abi.encodePacked(Utils.swapBytes4(uint32(len)), outcome.id, outcome.outcome.merkelization_hashes)
        );
    }

    struct MerklePathItem {
        bytes32 hash;
        uint8 direction; // 0 = left, 1 = right
    }

    function decodeMerklePathItem(Borsh.Data memory data) internal pure returns (MerklePathItem memory item) {
        item.hash = data.decodeBytes32();
        item.direction = data.decodeU8();
        require(item.direction < 2, "ProofDecoder: MerklePathItem direction should be 0 or 1");
    }

    struct MerklePath {
        MerklePathItem[] items;
    }

    function decodeMerklePath(Borsh.Data memory data) internal pure returns (MerklePath memory path) {
        path.items = new MerklePathItem[](data.decodeU32());
        for (uint i = 0; i < path.items.length; i++) {
            path.items[i] = data.decodeMerklePathItem();
        }
    }

    struct ExecutionOutcomeWithIdAndProof {
        MerklePath proof;
        bytes32 block_hash;
        ExecutionOutcomeWithId outcome_with_id;
    }

    function decodeExecutionOutcomeWithIdAndProof(Borsh.Data memory data)
        internal
        view
        returns (ExecutionOutcomeWithIdAndProof memory outcome)
    {
        outcome.proof = data.decodeMerklePath();
        outcome.block_hash = data.decodeBytes32();
        outcome.outcome_with_id = data.decodeExecutionOutcomeWithId();
    }
}

// File: contracts/INearProver.sol

pragma solidity ^0.6;

interface INearProver {
    function proveOutcome(bytes calldata proofData, uint64 blockHeight) external view returns (bool);
}

// File: contracts/NearProver.sol

pragma solidity ^0.6;







contract NearProver is INearProver, AdminControlled {
    using SafeMath for uint256;
    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;
    using ProofDecoder for Borsh.Data;

    INearBridge public bridge;

    constructor(
        INearBridge _bridge,
        address _admin,
        uint _pausedFlags
    ) public AdminControlled(_admin, _pausedFlags) {
        bridge = _bridge;
    }

    uint constant UNPAUSE_ALL = 0;
    uint constant PAUSED_VERIFY = 1;

    function proveOutcome(bytes memory proofData, uint64 blockHeight)
        public
        view
        override
        pausable(PAUSED_VERIFY)
        returns (bool)
    {
        Borsh.Data memory borsh = Borsh.from(proofData);
        ProofDecoder.FullOutcomeProof memory fullOutcomeProof = borsh.decodeFullOutcomeProof();
        borsh.done();

        bytes32 hash =
            _computeRoot(fullOutcomeProof.outcome_proof.outcome_with_id.hash, fullOutcomeProof.outcome_proof.proof);

        hash = sha256(abi.encodePacked(hash));

        hash = _computeRoot(hash, fullOutcomeProof.outcome_root_proof);

        require(
            hash == fullOutcomeProof.block_header_lite.inner_lite.outcome_root,
            "NearProver: outcome merkle proof is not valid"
        );

        bytes32 expectedBlockMerkleRoot = bridge.blockMerkleRoots(blockHeight);

        require(
            _computeRoot(fullOutcomeProof.block_header_lite.hash, fullOutcomeProof.block_proof) ==
                expectedBlockMerkleRoot,
            "NearProver: block proof is not valid"
        );

        return true;
    }

    function _computeRoot(bytes32 node, ProofDecoder.MerklePath memory proof) internal pure returns (bytes32 hash) {
        hash = node;
        for (uint i = 0; i < proof.items.length; i++) {
            ProofDecoder.MerklePathItem memory item = proof.items[i];
            if (item.direction == 0) {
                hash = sha256(abi.encodePacked(item.hash, hash));
            } else {
                hash = sha256(abi.encodePacked(hash, item.hash));
            }
        }
    }
}
