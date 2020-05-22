
// File: @openzeppelin/contracts/math/SafeMath.sol

pragma solidity ^0.5.0;

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
     * @dev Returns the addition of two unsigned integers, reverting on
     * overflow.
     *
     * Counterpart to Solidity's `+` operator.
     *
     * Requirements:
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
     * - Subtraction cannot overflow.
     */
    function sub(uint256 a, uint256 b) internal pure returns (uint256) {
        return sub(a, b, "SafeMath: subtraction overflow");
    }

    /**
     * @dev Returns the subtraction of two unsigned integers, reverting with custom message on
     * overflow (when the result is negative).
     *
     * Counterpart to Solidity's `-` operator.
     *
     * Requirements:
     * - Subtraction cannot overflow.
     *
     * _Available since v2.4.0._
     */
    function sub(uint256 a, uint256 b, string memory errorMessage) internal pure returns (uint256) {
        require(b <= a, errorMessage);
        uint256 c = a - b;

        return c;
    }

    /**
     * @dev Returns the multiplication of two unsigned integers, reverting on
     * overflow.
     *
     * Counterpart to Solidity's `*` operator.
     *
     * Requirements:
     * - Multiplication cannot overflow.
     */
    function mul(uint256 a, uint256 b) internal pure returns (uint256) {
        // Gas optimization: this is cheaper than requiring 'a' not being zero, but the
        // benefit is lost if 'b' is also tested.
        // See: https://github.com/OpenZeppelin/openzeppelin-contracts/pull/522
        if (a == 0) {
            return 0;
        }

        uint256 c = a * b;
        require(c / a == b, "SafeMath: multiplication overflow");

        return c;
    }

    /**
     * @dev Returns the integer division of two unsigned integers. Reverts on
     * division by zero. The result is rounded towards zero.
     *
     * Counterpart to Solidity's `/` operator. Note: this function uses a
     * `revert` opcode (which leaves remaining gas untouched) while Solidity
     * uses an invalid opcode to revert (consuming all remaining gas).
     *
     * Requirements:
     * - The divisor cannot be zero.
     */
    function div(uint256 a, uint256 b) internal pure returns (uint256) {
        return div(a, b, "SafeMath: division by zero");
    }

    /**
     * @dev Returns the integer division of two unsigned integers. Reverts with custom message on
     * division by zero. The result is rounded towards zero.
     *
     * Counterpart to Solidity's `/` operator. Note: this function uses a
     * `revert` opcode (which leaves remaining gas untouched) while Solidity
     * uses an invalid opcode to revert (consuming all remaining gas).
     *
     * Requirements:
     * - The divisor cannot be zero.
     *
     * _Available since v2.4.0._
     */
    function div(uint256 a, uint256 b, string memory errorMessage) internal pure returns (uint256) {
        // Solidity only automatically asserts when dividing by 0
        require(b > 0, errorMessage);
        uint256 c = a / b;
        // assert(a == b * c + a % b); // There is no case in which this doesn't hold

        return c;
    }

    /**
     * @dev Returns the remainder of dividing two unsigned integers. (unsigned integer modulo),
     * Reverts when dividing by zero.
     *
     * Counterpart to Solidity's `%` operator. This function uses a `revert`
     * opcode (which leaves remaining gas untouched) while Solidity uses an
     * invalid opcode to revert (consuming all remaining gas).
     *
     * Requirements:
     * - The divisor cannot be zero.
     */
    function mod(uint256 a, uint256 b) internal pure returns (uint256) {
        return mod(a, b, "SafeMath: modulo by zero");
    }

    /**
     * @dev Returns the remainder of dividing two unsigned integers. (unsigned integer modulo),
     * Reverts with custom message when dividing by zero.
     *
     * Counterpart to Solidity's `%` operator. This function uses a `revert`
     * opcode (which leaves remaining gas untouched) while Solidity uses an
     * invalid opcode to revert (consuming all remaining gas).
     *
     * Requirements:
     * - The divisor cannot be zero.
     *
     * _Available since v2.4.0._
     */
    function mod(uint256 a, uint256 b, string memory errorMessage) internal pure returns (uint256) {
        require(b != 0, errorMessage);
        return a % b;
    }
}

// File: @openzeppelin/contracts/GSN/Context.sol

pragma solidity ^0.5.0;

/*
 * @dev Provides information about the current execution context, including the
 * sender of the transaction and its data. While these are generally available
 * via msg.sender and msg.data, they should not be accessed in such a direct
 * manner, since when dealing with GSN meta-transactions the account sending and
 * paying for execution may not be the actual sender (as far as an application
 * is concerned).
 *
 * This contract is only required for intermediate, library-like contracts.
 */
contract Context {
    // Empty internal constructor, to prevent people from mistakenly deploying
    // an instance of this contract, which should be used via inheritance.
    constructor () internal { }
    // solhint-disable-previous-line no-empty-blocks

    function _msgSender() internal view returns (address payable) {
        return msg.sender;
    }

    function _msgData() internal view returns (bytes memory) {
        this; // silence state mutability warning without generating bytecode - see https://github.com/ethereum/solidity/issues/2691
        return msg.data;
    }
}

// File: @openzeppelin/contracts/ownership/Ownable.sol

pragma solidity ^0.5.0;

/**
 * @dev Contract module which provides a basic access control mechanism, where
 * there is an account (an owner) that can be granted exclusive access to
 * specific functions.
 *
 * This module is used through inheritance. It will make available the modifier
 * `onlyOwner`, which can be applied to your functions to restrict their use to
 * the owner.
 */
contract Ownable is Context {
    address private _owner;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    /**
     * @dev Initializes the contract setting the deployer as the initial owner.
     */
    constructor () internal {
        _owner = _msgSender();
        emit OwnershipTransferred(address(0), _owner);
    }

    /**
     * @dev Returns the address of the current owner.
     */
    function owner() public view returns (address) {
        return _owner;
    }

    /**
     * @dev Throws if called by any account other than the owner.
     */
    modifier onlyOwner() {
        require(isOwner(), "Ownable: caller is not the owner");
        _;
    }

    /**
     * @dev Returns true if the caller is the current owner.
     */
    function isOwner() public view returns (bool) {
        return _msgSender() == _owner;
    }

    /**
     * @dev Leaves the contract without owner. It will not be possible to call
     * `onlyOwner` functions anymore. Can only be called by the current owner.
     *
     * NOTE: Renouncing ownership will leave the contract without an owner,
     * thereby removing any functionality that is only available to the owner.
     */
    function renounceOwnership() public onlyOwner {
        emit OwnershipTransferred(_owner, address(0));
        _owner = address(0);
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`newOwner`).
     * Can only be called by the current owner.
     */
    function transferOwnership(address newOwner) public onlyOwner {
        _transferOwnership(newOwner);
    }

    /**
     * @dev Transfers ownership of the contract to a new account (`newOwner`).
     */
    function _transferOwnership(address newOwner) internal {
        require(newOwner != address(0), "Ownable: new owner is the zero address");
        emit OwnershipTransferred(_owner, newOwner);
        _owner = newOwner;
    }
}

// File: contracts/Borsh.sol

pragma solidity ^0.5.0;



library Borsh {

    using SafeMath for uint256;

    struct Data {
        uint256 offset;
        bytes raw;
    }

    function from(bytes memory data) internal pure returns(Data memory) {
        return Data({
            offset: 0,
            raw: data
        });
    }

    modifier shift(Data memory data, uint256 size) {
        require(data.raw.length >= data.offset + size, "Borsh: Out of range");
        _;
        data.offset += size;
    }

    function finished(Data memory data) internal pure returns(bool) {
        return data.offset == data.raw.length;
    }

    function peekKeccak256(Data memory data, uint256 length) internal pure returns(bytes32 res) {
        bytes memory ptr = data.raw;
        uint256 offset = data.offset;
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            res := keccak256(add(add(ptr, 32), offset), length)
        }
    }

    function decodeU8(Data memory data) internal pure shift(data, 1) returns(uint8 value) {
        value = uint8(data.raw[data.offset]);
    }

    function decodeI8(Data memory data) internal pure shift(data, 1) returns(int8 value) {
        value = int8(data.raw[data.offset]);
    }

    function decodeU16(Data memory data) internal pure returns(uint16 value) {
        value = uint16(decodeU8(data));
        value |= (uint16(decodeU8(data)) << 8);
    }

    function decodeI16(Data memory data) internal pure returns(int16 value) {
        value = int16(decodeI8(data));
        value |= (int16(decodeI8(data)) << 8);
    }

    function decodeU32(Data memory data) internal pure returns(uint32 value) {
        value = uint32(decodeU16(data));
        value |= (uint32(decodeU16(data)) << 16);
    }

    function decodeI32(Data memory data) internal pure returns(int32 value) {
        value = int32(decodeI16(data));
        value |= (int32(decodeI16(data)) << 16);
    }

    function decodeU64(Data memory data) internal pure returns(uint64 value) {
        value = uint64(decodeU32(data));
        value |= (uint64(decodeU32(data)) << 32);
    }

    function decodeI64(Data memory data) internal pure returns(int64 value) {
        value = int64(decodeI32(data));
        value |= (int64(decodeI32(data)) << 32);
    }

    function decodeU128(Data memory data) internal pure returns(uint128 value) {
        value = uint128(decodeU64(data));
        value |= (uint128(decodeU64(data)) << 64);
    }

    function decodeI128(Data memory data) internal pure returns(int128 value) {
        value = int128(decodeI64(data));
        value |= (int128(decodeI64(data)) << 64);
    }

    function decodeU256(Data memory data) internal pure returns(uint256 value) {
        value = uint256(decodeU128(data));
        value |= (uint256(decodeU128(data)) << 128);
    }

    function decodeI256(Data memory data) internal pure returns(int256 value) {
        value = int256(decodeI128(data));
        value |= (int256(decodeI128(data)) << 128);
    }

    function decodeBool(Data memory data) internal pure returns(bool value) {
        value = (decodeU8(data) != 0);
    }

    function decodeBytes(Data memory data) internal pure returns(bytes memory value) {
        value = new bytes(decodeU32(data));
        for (uint i = 0; i < value.length; i++) {
            value[i] = byte(decodeU8(data));
        }
    }

    function decodeBytes32(Data memory data) internal pure shift(data, 32) returns(bytes32 value) {
        bytes memory raw = data.raw;
        uint256 offset = data.offset;
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            value := mload(add(add(raw, 32), offset))
        }
    }

    struct PublicKey {
        uint256 x;
        uint256 y;
    }

    function decodePublicKey(Borsh.Data memory data) internal pure returns(PublicKey memory key) {
        key.x = decodeU256(data);
        key.y = decodeU256(data);
    }

    struct ED25519PublicKey {
        bytes32 xy;
    }

    function decodeED25519PublicKey(Borsh.Data memory data) internal pure returns(ED25519PublicKey memory key) {
        key.xy = decodeBytes32(data);
    }

    struct Signature {
        bytes32 r;
        bytes32 s;
        uint8 v;
    }

    function decodeSignature(Borsh.Data memory data) internal pure returns(Signature memory sig) {
        sig.r = decodeBytes32(data);
        sig.s = decodeBytes32(data);
        sig.v = decodeU8(data);
    }

    struct ED25519Signature {
        bytes32[2] rs;
    }

    function decodeED25519Signature(Borsh.Data memory data) internal pure returns(ED25519Signature memory sig) {
        sig.rs[0] = decodeBytes32(data);
        sig.rs[1] = decodeBytes32(data);
    }
}

// File: contracts/NearDecoder.sol

pragma solidity ^0.5.0;




library NearDecoder {

    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;

    struct ValidatorStake {
        string account_id;
        Borsh.ED25519PublicKey public_key;
        uint128 stake;
    }

    function decodeValidatorStake(Borsh.Data memory data) internal pure returns(ValidatorStake memory validatorStake) {
        validatorStake.account_id = string(data.decodeBytes());
        validatorStake.public_key = data.decodeED25519PublicKey();
        validatorStake.stake = data.decodeU128();
    }

    struct OptionalValidatorStakes {
        bool none;
        ValidatorStake[] validatorStakes;

        bytes32 hash; // Additional computable element
    }

    function decodeOptionalValidatorStakes(Borsh.Data memory data) internal pure returns(OptionalValidatorStakes memory stakes) {
        stakes.none = (data.decodeU8() == 0);
        if (!stakes.none) {
            stakes.validatorStakes = new ValidatorStake[](data.decodeU32());

            bytes memory bps_data = "";
            for (uint i = 0; i < stakes.validatorStakes.length; i++) {
                stakes.validatorStakes[i] = data.decodeValidatorStake();

                bps_data = abi.encodePacked(
                    bps_data,
                    sha256(abi.encodePacked(stakes.validatorStakes[i].account_id)),
                    sha256(abi.encodePacked(stakes.validatorStakes[i].public_key.xy)),
                    sha256(abi.encodePacked(stakes.validatorStakes[i].stake))
                );
            }

            // Calculate keccak256(borsh(bps))
            stakes.hash = sha256(bps_data);
        }
    }

    struct OptionalED25519Signature {
        bool none;
        Borsh.ED25519Signature signature;
    }

    function decodeOptionalED25519Signature(Borsh.Data memory data) internal pure returns(OptionalED25519Signature memory sig) {
        sig.none = (data.decodeU8() == 0);
        if (!sig.none) {
            sig.signature = data.decodeED25519Signature();
        }
    }

    function decodeOptionalED25519Signatures(Borsh.Data memory data) internal pure returns(OptionalED25519Signature[] memory sigs) {
        sigs = new OptionalED25519Signature[](data.decodeU32());
        for (uint  i = 0; i < sigs.length; i++) {
            sigs[i] = data.decodeOptionalED25519Signature();
        }
    }

    struct LightClientBlock {
        bytes32 prev_block_hash;
        bytes32 next_block_inner_hash;
        BlockHeaderInnerLite inner_lite;
        bytes32 inner_rest_hash;
        OptionalValidatorStakes next_bps;
        OptionalED25519Signature[] approvals_next; // TODO: delete
        OptionalED25519Signature[] approvals_after_next;
    }

    function decodeLightClientBlock(Borsh.Data memory data) internal pure returns(LightClientBlock memory header) {
        header.prev_block_hash = data.decodeBytes32();
        header.next_block_inner_hash = data.decodeBytes32();
        header.inner_lite = data.decodeBlockHeaderInnerLite();
        header.inner_rest_hash = data.decodeBytes32();
        header.next_bps = data.decodeOptionalValidatorStakes();
        header.approvals_next = data.decodeOptionalED25519Signatures();
        header.approvals_after_next = data.decodeOptionalED25519Signatures();
    }

    struct BlockHeaderInnerLite {
        uint64 height;              /// Height of this block since the genesis block (height 0).
        bytes32 epoch_id;           /// Epoch start hash of this block's epoch. Used for retrieving validator information
        bytes32 next_epoch_id;
        bytes32 prev_state_root;    /// Root hash of the state at the previous block.
        bytes32 outcome_root;       /// Root of the outcomes of transactions and receipts.
        uint64 timestamp;           /// Timestamp at which the block was built.
        bytes32 next_bp_hash;       /// Hash of the next epoch block producers set

        bytes32 hash; // Additional computable element
    }

    function decodeBlockHeaderInnerLite(Borsh.Data memory data) internal pure returns(BlockHeaderInnerLite memory header) {
        header.hash = data.peekKeccak256(176);
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
        bytes32 successReceiptId;   /// The final action of the receipt returned a promise or the signed
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
        bytes32[] receipt_ids;  /// Receipt IDs generated by this transaction or receipt.
        uint64 gas_burnt;       /// The amount of the gas burnt by the given transaction or receipt.
        ExecutionStatus status; /// Execution status. Contains the result in case of successful execution.
    }

    function decodeExecutionOutcome(Borsh.Data memory data) internal pure returns(ExecutionOutcome memory outcome) {
        outcome.logs = new bytes[](data.decodeU32());
        for (uint i = 0; i < outcome.logs.length; i++) {
            outcome.logs[i] = data.decodeBytes();
        }
        outcome.receipt_ids = new bytes32[](data.decodeU32());
        for (uint i = 0; i < outcome.receipt_ids.length; i++) {
            outcome.receipt_ids[i] = data.decodeBytes32();
        }
        outcome.gas_burnt = data.decodeU64();
        outcome.status = data.decodeExecutionStatus();
    }

    struct ExecutionOutcomeWithId {
        bytes32 id; /// The transaction hash or the receipt ID.
        ExecutionOutcome outcome;
    }

    function decodeExecutionOutcomeWithId(Borsh.Data memory data) internal pure returns(ExecutionOutcomeWithId memory outcome) {
        outcome.id = data.decodeBytes32();
        outcome.outcome = data.decodeExecutionOutcome();
    }

    struct MerklePathItem {
        bytes32 hash;
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
        bytes32 block_hash;
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

// File: contracts/ED25519.sol

pragma solidity ^0.5.0;



library ED25519 {
    function verify(bytes32 hash, bytes32 publicKey, bytes32[2] memory signature) internal view returns(bool success) {
        return true;
        // bytes32[4] memory data = [
        //     hash,
        //     publicKey,
        //     signature[0],
        //     signature[1]
        // ];

        // bool success;
        // uint32[1] memory result = [uint32(7)];
        // // solium-disable-next-line security/no-inline-assembly
        // assembly{
        //     success := staticcall(gas, 0x9, add(data, 32), 128, result, 4)
        //     switch success
        //         case 0 { revert(0, returndatasize) }
        // }

        // return result[0] == 0;
    }
}

// Using formulas from https://hyperelliptic.org/EFD/g1p/auto-twisted-projective.html
// and constants from https://tools.ietf.org/html/draft-josefsson-eddsa-ed25519-03


// https://ed25519.cr.yp.to/python/ed25519.py
// library Ed25519 {
//     uint constant q = 2 ** 255 - 19;
//     uint constant d = 37095705934669439343138083508754565189542113879843219016388785533085940283555;
//                       // = -(121665/121666)
//     uint constant Bx = 15112221349535400772501151409588531511454012693041857206046113283949847762202;
//     uint constant By = 46316835694926478169428394003475163141307993866256225615783033603165251855960;

//     struct Point {
//         uint x;
//         uint y;
//         uint z;
//     }

//     struct Scratchpad {
//         uint a;
//         uint b;
//         uint c;
//         uint d;
//         uint e;
//         uint f;
//         uint g;
//         uint h;
//     }

//     function submod(uint256 a, uint256 b, uint256 m) internal pure returns(uint256) {
//         return (a + m - b) % m;
//     }

//     function isPubKey(uint[2] memory P) public view returns(bool) {
//         uint n = d;
//         uint p = q;
//         uint256 x = P[0];
//         uint256 y = P[1];
//         uint256 xx = mulmod(x, x, n);
//         uint256 yy = mulmod(y, y, n);
//         return submod(yy - xx - 1 - mulmod(n, mulmod(xx, yy, n), n)) % p == 0
//     }

//     function validateSignature(bytes32 message, uint[2] memory rs, uint[2] memory Q) public view returns (bool) {
//         uint n = d;
//         uint p = q;
//         if(rs[0] == 0 || rs[0] >= n || rs[1] == 0 || rs[1] > n/2)
//             return false;
//         if (!isPubKey(Q))
//             return false;

//         uint sInv = inv(rs[1]);
//         uint[3] memory u1G = _mul(mulmod(uint(message), sInv, n), [Gx, Gy]);
//         uint[3] memory u2Q = _mul(mulmod(rs[0], sInv, n), Q);
//         uint[3] memory P = _add(u1G, u2Q);

//         if (P[2] == 0)
//             return false;

//         uint Px = inv(P[2]); // need Px/Pz^2
//         Px = mulmod(P[0], mulmod(Px, Px, p), p);
//         return Px % n == rs[0];
//     }

//     function inv(uint a) internal view returns (uint invA) {
//         uint e = q - 2;
//         uint m = q;

//         // use bigModExp precompile
//         assembly {
//             let p := mload(0x40)
//             mstore(p, 0x20)
//             mstore(add(p, 0x20), 0x20)
//             mstore(add(p, 0x40), 0x20)
//             mstore(add(p, 0x60), a)
//             mstore(add(p, 0x80), e)
//             mstore(add(p, 0xa0), m)
//             if iszero(staticcall(not(0), 0x05, p, 0xc0, p, 0x20)) {
//                 revert(0, 0)
//             }
//             invA := mload(p)
//         }
//     }

//     function ecAdd(Point memory p1,
//                    Point memory p2) internal pure returns (Point memory p3) {
//         Scratchpad memory tmp;

//         tmp.a = mulmod(p1.z, p2.z, q);
//         tmp.b = mulmod(tmp.a, tmp.a, q);
//         tmp.c = mulmod(p1.x, p2.x, q);
//         tmp.d = mulmod(p1.y, p2.y, q);
//         tmp.e = mulmod(d, mulmod(tmp.c, tmp.d, q), q);
//         tmp.f = addmod(tmp.b, q - tmp.e, q);
//         tmp.g = addmod(tmp.b, tmp.e, q);
//         p3.x = mulmod(mulmod(tmp.a, tmp.f, q),
//                       addmod(addmod(mulmod(addmod(p1.x, p1.y, q),
//                                            addmod(p2.x, p2.y, q), q),
//                                     q - tmp.c, q), q - tmp.d, q), q);
//         p3.y = mulmod(mulmod(tmp.a, tmp.g, q),
//                       addmod(tmp.d, tmp.c, q), q);
//         p3.z = mulmod(tmp.f, tmp.g, q);
//     }

//     function ecDouble(Point memory p1) internal pure returns (Point memory p2) {
//         Scratchpad memory tmp;

//         tmp.a = addmod(p1.x, p1.y, q);
//         tmp.b = mulmod(tmp.a, tmp.a, q);
//         tmp.c = mulmod(p1.x, p1.x, q);
//         tmp.d = mulmod(p1.y, p1.y, q);
//         tmp.e = q - tmp.c;
//         tmp.f = addmod(tmp.e, tmp.d, q);
//         tmp.h = mulmod(p1.z, p1.z, q);
//         tmp.g = addmod(tmp.f, q - mulmod(2, tmp.h, q), q);
//         p2.x = mulmod(addmod(addmod(tmp.b, q - tmp.c, q), q - tmp.d, q),
//                       tmp.g, q);
//         p2.y = mulmod(tmp.f, addmod(tmp.e, q - tmp.d, q), q);
//         p2.z = mulmod(tmp.f, tmp.g, q);
//     }

//     function scalarMultBase(uint s) public view returns (uint, uint) {
//         Point memory b;
//         Point memory result;
//         b.x = Bx;
//         b.y = By;
//         b.z = 1;
//         result.x = 0;
//         result.y = 1;
//         result.z = 1;

//         while (s > 0) {
//             if (s & 1 == 1) { result = ecAdd(result, b); }
//             s = s >> 1;
//             b = ecDouble(b);
//         }

//         uint invZ = inv(result.z);
//         result.x = mulmod(result.x, invZ, q);
//         result.y = mulmod(result.y, invZ, q);

//         return (result.x, result.y);
//     }
// }

// File: contracts/NearBridge.sol

pragma solidity ^0.5.0;
pragma experimental ABIEncoderV2; // solium-disable-line no-experimental






contract NearBridge is Ownable {
    using SafeMath for uint256;
    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;

    struct State {
        uint256 height;
        bytes32 epochId;
        bytes32 nextEpochId;
        address submitter;
        uint256 validAfter;
        bytes32 hash;
    }

    uint256 constant public LOCK_ETH_AMOUNT = 1 ether;
    uint256 constant public LOCK_DURATION = 1 hours;

    State public last;
    State public prev;
    mapping(uint256 => bytes32) public blockHashes;
    mapping(address => uint256) public balanceOf;

    event BlockHashAdded(
        uint256 indexed height,
        bytes32 blockHash
    );

    constructor(bytes32 firstEpochId, bytes32 firstNextEpochId) public {
        last.epochId = firstEpochId;
        last.nextEpochId = firstNextEpochId;
    }

    function deposit() public payable {
        require(msg.value == LOCK_ETH_AMOUNT && balanceOf[msg.sender] == 0);
        balanceOf[msg.sender] = balanceOf[msg.sender].add(msg.value);
    }

    function withdraw() public {
        balanceOf[msg.sender] = balanceOf[msg.sender].sub(LOCK_ETH_AMOUNT);
        msg.sender.transfer(LOCK_ETH_AMOUNT);
    }

    function validate(address user, address payable receiver, bytes memory data) public {
        require(last.hash == keccak256(data), "Data did not match");
        require(block.timestamp < last.validAfter, "Lock period already passed");

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh.decodeLightClientBlock();
        bytes32 nearBlockHash = hash(nearBlock);
        bytes32 nearBlockNextHash = nextHash(nearBlock, nearBlockHash);

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
                nearBlockHash,
                nearBlock.approvals_next,
                nearBlock.next_bps.validatorStakes
            ),
            "NearBridge: Less than 2/3 voted by the next block"
        );
        require(
            _checkValidatorSignatures(
                totalStake,
                nearBlockNextHash,
                nearBlock.approvals_after_next,
                nearBlock.next_bps.validatorStakes
            ),
            "NearBridge: Less than 2/3 voted by the block after next"
        );

        // Pay reward
        balanceOf[user] = balanceOf[user].sub(LOCK_ETH_AMOUNT);
        receiver.transfer(LOCK_ETH_AMOUNT);

        // Erase last state
        delete blockHashes[last.height];
        last = prev;
    }

    function addLightClientBlock(bytes memory data) public payable {
        require(balanceOf[msg.sender] >= LOCK_ETH_AMOUNT, "Balance is not enough");
        require(block.timestamp >= last.validAfter, "Wait until last block become valid");

        Borsh.Data memory borsh = Borsh.from(data);
        NearDecoder.LightClientBlock memory nearBlock = borsh.decodeLightClientBlock();
        require(borsh.finished(), "NearBridge: only light client block should be passed");
        bytes32 nearBlockHash = hash(nearBlock);

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
                "NearBridge: Next bps should no be None"
            );
        }

        // 4. approvals_next and approvals_after_next contain signatures that check out against the block producers for the epoch of the block
        // 5. The signatures present in both approvals_next and approvals_after_next correspond to more than 2/3 of the total stake
        // uint256 totalStake = 0;
        // for (uint i = 0; i < nearBlock.next_bps.validatorStakes.length; i++) {
        //     totalStake = totalStake.add(
        //         nearBlock.next_bps.validatorStakes[i].stake
        //     );
        // }
        // require(
        //     _checkValidatorSignatures(
        //         totalStake,
        //         nearBlockHash,
        //         nearBlock.approvals_next,
        //         nearBlock.next_bps.validatorStakes
        //     ),
        //     "NearBridge: Less than 2/3 voted by the next block"
        // );
        // require(
        //     _checkValidatorSignatures(
        //         totalStake,
        //         nearBlockNextHash,
        //         nearBlock.approvals_after_next,
        //         nearBlock.next_bps.validatorStakes
        //     ),
        //     "NearBridge: Less than 2/3 voted by the block after next"
        // );

        // 6. If next_bps is not none, sha256(borsh(next_bps)) corresponds to the next_bp_hash in inner_lite.
        if (!nearBlock.next_bps.none) {
            require(
                nearBlock.next_bps.hash == nearBlock.inner_lite.next_bp_hash,
                "NearBridge: Hash of block producers do not match"
            );
        }

        // Finish:
        prev = last;
        last = State({
            height: nearBlock.inner_lite.height,
            epochId: nearBlock.inner_lite.epoch_id,
            nextEpochId: nearBlock.inner_lite.next_epoch_id,
            submitter: msg.sender,
            validAfter: block.timestamp.add(LOCK_DURATION),
            hash: keccak256(data)
        });
        blockHashes[nearBlock.inner_lite.height] = nearBlockHash;
        emit BlockHashAdded(
            last.height,
            blockHashes[last.height]
        );
    }

    function _checkValidatorSignatures(
        uint256 totalStake,
        bytes32 next_block_inner_hash,
        NearDecoder.OptionalED25519Signature[] memory approvals,
        NearDecoder.ValidatorStake[] memory validatorStakes
    ) internal view returns(bool) {
        uint256 votedFor = 0;
        uint256 votedAgainst = 0;
        for (uint i = 0; i < approvals.length; i++) {
            if (approvals[i].none) {
                votedAgainst = votedAgainst.add(validatorStakes[i].stake);
            } else {
                require(
                    ED25519.verify(
                        next_block_inner_hash,
                        validatorStakes[i].public_key.xy,
                        approvals[i].signature.rs
                    ),
                    "NearBridge: Validator signature is not valid"
                );
                votedFor = votedFor.add(validatorStakes[i].stake);
            }

            if (votedFor > totalStake.mul(2).div(3)) {
                return true;
            }
            if (votedAgainst >= totalStake.mul(1).div(3)) {
                return false;
            }
        }

        revert("NearBridge: Should never be reached");
    }

    function hash(NearDecoder.LightClientBlock memory nearBlock) public view returns(bytes32) {
        return keccak256(abi.encodePacked(
            nearBlock.prev_block_hash,
            keccak256(abi.encodePacked(
                nearBlock.inner_lite.hash,
                nearBlock.inner_rest_hash
            ))
        ));
    }

    function nextHash(NearDecoder.LightClientBlock memory nearBlock, bytes32 currentHash) public view returns(bytes32) {
        return keccak256(abi.encodePacked(
            currentHash,
            nearBlock.next_block_inner_hash
        ));
    }
}
