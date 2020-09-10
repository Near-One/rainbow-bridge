
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

// File: contracts/INearBridge.sol

pragma solidity ^0.5.0;


interface INearBridge {
    event BlockHashAdded(
        uint64 indexed height,
        bytes32 blockHash
    );

    event BlockHashReverted(
        uint64 indexed height,
        bytes32 blockHash
    );

    function blockHashes(uint64 blockNumber) external view returns(bytes32);
    function blockMerkleRoots(uint64 blockNumber) external view returns(bytes32);

    function balanceOf(address wallet) external view returns(uint256);
    function deposit() external payable;
    function withdraw() external;

    function initWithValidators(bytes calldata initialValidators) external;
    function initWithBlock(bytes calldata data) external;
    function addLightClientBlock(bytes calldata data) external payable;
    function challenge(address payable receiver, uint256 signatureIndex) external;
    function checkBlockProducerSignatureInHead(uint256 signatureIndex) external view returns(bool);
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
        return bytesKeccak256(data.raw, data.offset, length);
    }

    function bytesKeccak256(bytes memory ptr, uint256 offset, uint256 length) internal pure returns(bytes32 res) {
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            res := keccak256(add(add(ptr, 32), offset), length)
        }
    }

    function peekSha256(Data memory data, uint256 length) internal view returns(bytes32) {
        return bytesSha256(data.raw, data.offset, length);
    }

    function bytesSha256(bytes memory ptr, uint256 offset, uint256 length) internal view returns(bytes32) {
        bytes32[1] memory result;
        // solium-disable-next-line security/no-inline-assembly
        assembly {
            pop(staticcall(gas, 0x02, add(add(ptr, 32), offset), length, result, 32))
        }
        return result[0];
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

    function decodeBytes20(Data memory data) internal pure returns(bytes20 value) {
        for (uint i = 0; i < 20; i++) {
            value |= bytes20(byte(decodeU8(data)) & 0xFF) >> (i * 8);
        }
    }

    // Public key

    struct SECP256K1PublicKey {
        uint256 x;
        uint256 y;
    }

    function decodeSECP256K1PublicKey(Borsh.Data memory data) internal pure returns(SECP256K1PublicKey memory key) {
        key.x = decodeU256(data);
        key.y = decodeU256(data);
    }

    struct ED25519PublicKey {
        bytes32 xy;
    }

    function decodeED25519PublicKey(Borsh.Data memory data) internal pure returns(ED25519PublicKey memory key) {
        key.xy = decodeBytes32(data);
    }

    // Signature

    struct SECP256K1Signature {
        bytes32 r;
        bytes32 s;
        uint8 v;
    }

    function decodeSECP256K1Signature(Borsh.Data memory data) internal pure returns(SECP256K1Signature memory sig) {
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

    struct PublicKey {
        uint8 enumIndex;

        Borsh.ED25519PublicKey ed25519;
        Borsh.SECP256K1PublicKey secp256k1;
    }

    function decodePublicKey(Borsh.Data memory data) internal pure returns(PublicKey memory key) {
        key.enumIndex = data.decodeU8();

        if (key.enumIndex == 0) {
            key.ed25519 = data.decodeED25519PublicKey();
        }
        else if (key.enumIndex == 1) {
            key.secp256k1 = data.decodeSECP256K1PublicKey();
        }
        else {
            revert("NearBridge: Only ED25519 and SECP256K1 public keys are supported");
        }
    }

    struct ValidatorStake {
        string account_id;
        PublicKey public_key;
        uint128 stake;
    }

    function decodeValidatorStake(Borsh.Data memory data) internal pure returns(ValidatorStake memory validatorStake) {
        validatorStake.account_id = string(data.decodeBytes());
        validatorStake.public_key = data.decodePublicKey();
        validatorStake.stake = data.decodeU128();
    }

    struct OptionalValidatorStakes {
        bool none;

        ValidatorStake[] validatorStakes;
        bytes32 hash; // Additional computable element
    }

    function decodeOptionalValidatorStakes(Borsh.Data memory data) internal view returns(OptionalValidatorStakes memory stakes) {
        stakes.none = (data.decodeU8() == 0);
        if (!stakes.none) {
            uint256 start = data.offset;

            stakes.validatorStakes = new ValidatorStake[](data.decodeU32());
            for (uint i = 0; i < stakes.validatorStakes.length; i++) {
                stakes.validatorStakes[i] = data.decodeValidatorStake();
            }

            uint256 stop = data.offset;
            data.offset = start;
            stakes.hash = data.peekSha256(stop - start);
            data.offset = stop;
        }
    }

    struct Signature {
        uint8 enumIndex;

        Borsh.ED25519Signature ed25519;
        Borsh.SECP256K1Signature secp256k1;
    }

    function decodeSignature(Borsh.Data memory data) internal pure returns(Signature memory sig) {
        sig.enumIndex = data.decodeU8();

        if (sig.enumIndex == 0) {
            sig.ed25519 = data.decodeED25519Signature();
        }
        else if (sig.enumIndex == 1) {
            sig.secp256k1 = data.decodeSECP256K1Signature();
        }
        else {
            revert("NearBridge: Only ED25519 and SECP256K1 signatures are supported");
        }
    }

    struct OptionalSignature {
        bool none;
        Signature signature;
    }

    function decodeOptionalSignature(Borsh.Data memory data) internal pure returns(OptionalSignature memory sig) {
        sig.none = (data.decodeU8() == 0);
        if (!sig.none) {
            sig.signature = data.decodeSignature();
        }
    }

    struct LightClientBlock {
        bytes32 prev_block_hash;
        bytes32 next_block_inner_hash;
        BlockHeaderInnerLite inner_lite;
        bytes32 inner_rest_hash;
        OptionalValidatorStakes next_bps;
        OptionalSignature[] approvals_after_next;

        bytes32 hash;
        bytes32 next_hash;
    }

    struct InitialValidators {
        ValidatorStake[] validator_stakes;
    }

    function decodeInitialValidators(Borsh.Data memory data) internal view returns(InitialValidators memory validators) {
        validators.validator_stakes = new ValidatorStake[](data.decodeU32());
        for (uint i = 0; i < validators.validator_stakes.length; i++) {
            validators.validator_stakes[i] = data.decodeValidatorStake();
        }
    }

    function decodeLightClientBlock(Borsh.Data memory data) internal view returns(LightClientBlock memory header) {
        header.prev_block_hash = data.decodeBytes32();
        header.next_block_inner_hash = data.decodeBytes32();
        header.inner_lite = data.decodeBlockHeaderInnerLite();
        header.inner_rest_hash = data.decodeBytes32();
        header.next_bps = data.decodeOptionalValidatorStakes();

        header.approvals_after_next = new OptionalSignature[](data.decodeU32());
        for (uint  i = 0; i < header.approvals_after_next.length; i++) {
            header.approvals_after_next[i] = data.decodeOptionalSignature();
        }

        header.hash = sha256(abi.encodePacked(
            sha256(abi.encodePacked(
                header.inner_lite.hash,
                header.inner_rest_hash
            )),
            header.prev_block_hash
        ));

        header.next_hash = sha256(abi.encodePacked(
            header.next_block_inner_hash,
            header.hash
        ));
    }

    struct BlockHeaderInnerLite {
        uint64 height;              /// Height of this block since the genesis block (height 0).
        bytes32 epoch_id;           /// Epoch start hash of this block's epoch. Used for retrieving validator information
        bytes32 next_epoch_id;
        bytes32 prev_state_root;    /// Root hash of the state at the previous block.
        bytes32 outcome_root;       /// Root of the outcomes of transactions and receipts.
        uint64 timestamp;           /// Timestamp at which the block was built.
        bytes32 next_bp_hash;       /// Hash of the next epoch block producers set
        bytes32 block_merkle_root;

        bytes32 hash; // Additional computable element
    }

    function decodeBlockHeaderInnerLite(Borsh.Data memory data) internal view returns(BlockHeaderInnerLite memory header) {
        header.hash = data.peekSha256(208);
        header.height = data.decodeU64();
        header.epoch_id = data.decodeBytes32();
        header.next_epoch_id = data.decodeBytes32();
        header.prev_state_root = data.decodeBytes32();
        header.outcome_root = data.decodeBytes32();
        header.timestamp = data.decodeU64();
        header.next_bp_hash = data.decodeBytes32();
        header.block_merkle_root = data.decodeBytes32();
    }
}

// File: contracts/Ed25519.sol

pragma solidity ^0.5.0;


// https://gist.github.com/abacabadabacaba/cb927b5ebe6db63f00b98902503ffc76
// https://gist.github.com/abacabadabacaba/9c395588c455ca1f7dccfa853d8fd56d
// https://play.golang.org/p/obqA-XDYzdm
contract Ed25519 {
    // Computes (v^(2^250-1), v^11) mod p
    function pow22501(uint256 v) private pure returns (uint256 p22501, uint256 p11) {
        p11 = mulmod(v, v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(p11, p11, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(mulmod(p22501, p22501, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p11 = mulmod(p22501, p11, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(mulmod(p11, p11, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), p22501, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        uint256 a = mulmod(p22501, p22501, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(p22501, p22501, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        uint256 b = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(p22501, p22501, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
    }

    function check(bytes32 k, bytes32 r, bytes32 s, bytes32 m1, bytes9 m2) public pure returns (bool) {
        uint256 hh;
        // Step 1: compute SHA-512(R, A, M)
        {
            uint256[5][16] memory kk = [[uint256(0x428a2f98_d728ae22), uint256(0xe49b69c1_9ef14ad2), uint256(0x27b70a85_46d22ffc), uint256(0x19a4c116_b8d2d0c8), uint256(0xca273ece_ea26619c)], [uint256(0x71374491_23ef65cd), uint256(0xefbe4786_384f25e3), uint256(0x2e1b2138_5c26c926), uint256(0x1e376c08_5141ab53), uint256(0xd186b8c7_21c0c207)], [uint256(0xb5c0fbcf_ec4d3b2f), uint256(0xfc19dc6_8b8cd5b5), uint256(0x4d2c6dfc_5ac42aed), uint256(0x2748774c_df8eeb99), uint256(0xeada7dd6_cde0eb1e)], [uint256(0xe9b5dba5_8189dbbc), uint256(0x240ca1cc_77ac9c65), uint256(0x53380d13_9d95b3df), uint256(0x34b0bcb5_e19b48a8), uint256(0xf57d4f7f_ee6ed178)], [uint256(0x3956c25b_f348b538), uint256(0x2de92c6f_592b0275), uint256(0x650a7354_8baf63de), uint256(0x391c0cb3_c5c95a63), uint256(0x6f067aa_72176fba)], [uint256(0x59f111f1_b605d019), uint256(0x4a7484aa_6ea6e483), uint256(0x766a0abb_3c77b2a8), uint256(0x4ed8aa4a_e3418acb), uint256(0xa637dc5_a2c898a6)], [uint256(0x923f82a4_af194f9b), uint256(0x5cb0a9dc_bd41fbd4), uint256(0x81c2c92e_47edaee6), uint256(0x5b9cca4f_7763e373), uint256(0x113f9804_bef90dae)], [uint256(0xab1c5ed5_da6d8118), uint256(0x76f988da_831153b5), uint256(0x92722c85_1482353b), uint256(0x682e6ff3_d6b2b8a3), uint256(0x1b710b35_131c471b)], [uint256(0xd807aa98_a3030242), uint256(0x983e5152_ee66dfab), uint256(0xa2bfe8a1_4cf10364), uint256(0x748f82ee_5defb2fc), uint256(0x28db77f5_23047d84)], [uint256(0x12835b01_45706fbe), uint256(0xa831c66d_2db43210), uint256(0xa81a664b_bc423001), uint256(0x78a5636f_43172f60), uint256(0x32caab7b_40c72493)], [uint256(0x243185be_4ee4b28c), uint256(0xb00327c8_98fb213f), uint256(0xc24b8b70_d0f89791), uint256(0x84c87814_a1f0ab72), uint256(0x3c9ebe0a_15c9bebc)], [uint256(0x550c7dc3_d5ffb4e2), uint256(0xbf597fc7_beef0ee4), uint256(0xc76c51a3_0654be30), uint256(0x8cc70208_1a6439ec), uint256(0x431d67c4_9c100d4c)], [uint256(0x72be5d74_f27b896f), uint256(0xc6e00bf3_3da88fc2), uint256(0xd192e819_d6ef5218), uint256(0x90befffa_23631e28), uint256(0x4cc5d4be_cb3e42b6)], [uint256(0x80deb1fe_3b1696b1), uint256(0xd5a79147_930aa725), uint256(0xd6990624_5565a910), uint256(0xa4506ceb_de82bde9), uint256(0x597f299c_fc657e2a)], [uint256(0x9bdc06a7_25c71235), uint256(0x6ca6351_e003826f), uint256(0xf40e3585_5771202a), uint256(0xbef9a3f7_b2c67915), uint256(0x5fcb6fab_3ad6faec)], [uint256(0xc19bf174_cf692694), uint256(0x14292967_0a0e6e70), uint256(0x106aa070_32bbd1b8), uint256(0xc67178f2_e372532b), uint256(0x6c44198c_4a475817)]];
            uint256 w0 = (uint256(r) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_ffffffff_ffffffff) | ((uint256(r) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) | ((uint256(r) & 0xffffffff_ffffffff_00000000_00000000) << 64);
            uint256 w1 = (uint256(k) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_ffffffff_ffffffff) | ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) | ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000) << 64);
            uint256 w2 = (uint256(m1) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_ffffffff_ffffffff) | ((uint256(m1) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) | ((uint256(m1) & 0xffffffff_ffffffff_00000000_00000000) << 64);
            uint256 w3 = (uint256(bytes32(m2)) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_00000000_00000000) | ((uint256(bytes32(m2)) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) | 0x800000_00000000_00000000_00000348;
            uint256 a = 0x6a09e667_f3bcc908;
            uint256 b = 0xbb67ae85_84caa73b;
            uint256 c = 0x3c6ef372_fe94f82b;
            uint256 d = 0xa54ff53a_5f1d36f1;
            uint256 e = 0x510e527f_ade682d1;
            uint256 f = 0x9b05688c_2b3e6c1f;
            uint256 g = 0x1f83d9ab_fb41bd6b;
            uint256 h = 0x5be0cd19_137e2179;
            for (uint256 i = 0;; i++) {
                // Round 16 * i
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[0][i];
                    temp1 += w0 >> 192;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 1
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[1][i];
                    temp1 += w0 >> 64;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 2
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[2][i];
                    temp1 += w0 >> 128;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 3
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[3][i];
                    temp1 += w0;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 4
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[4][i];
                    temp1 += w1 >> 192;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 5
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[5][i];
                    temp1 += w1 >> 64;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 6
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[6][i];
                    temp1 += w1 >> 128;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 7
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[7][i];
                    temp1 += w1;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 8
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[8][i];
                    temp1 += w2 >> 192;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 9
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[9][i];
                    temp1 += w2 >> 64;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 10
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[10][i];
                    temp1 += w2 >> 128;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 11
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[11][i];
                    temp1 += w2;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 12
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[12][i];
                    temp1 += w3 >> 192;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 13
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[13][i];
                    temp1 += w3 >> 64;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 14
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[14][i];
                    temp1 += w3 >> 128;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                // Round 16 * i + 15
                {
                    uint256 temp1;
                    uint256 temp2;
                    e &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = e | (e << 64);
                        uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                        uint256 ch = (e & (f ^ g)) ^ g;
                        temp1 = h + s1 + ch;
                    }
                    temp1 += kk[15][i];
                    temp1 += w3;
                    a &= 0xffffffff_ffffffff;
                    {
                        uint256 ss = a | (a << 64);
                        uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                        uint256 maj = (a & (b | c)) | (b & c);
                        temp2 = s0 + maj;
                    }
                    h = g;
                    g = f;
                    f = e;
                    e = d + temp1;
                    d = c;
                    c = b;
                    b = a;
                    a = temp1 + temp2;
                }
                if (i == 4) {
                    break;
                }
                // Message expansion
                uint256 t0 = w0;
                uint256 t1 = w1;
                {
                    uint256 t2 = w2;
                    uint256 t3 = w3;
                    {
                        uint256 n1 = t0 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        n1 += ((t2 & 0xffffffff_ffffffff_00000000_00000000) << 128) | ((t2 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                        {
                            uint256 u1 = ((t0 & 0xffffffff_ffffffff_00000000_00000000) << 64) | ((t0 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                            uint256 uu1 = u1 | (u1 << 64);
                            n1 += ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        {
                            uint256 v1 = t3 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            uint256 vv1 = v1 | (v1 << 64);
                            n1 += ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        uint256 n2 = t0 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        n2 += ((t2 & 0xffffffff_ffffffff) << 128) | (t3 >> 192);
                        {
                            uint256 u2 = ((t0 & 0xffffffff_ffffffff) << 128) | (t1 >> 192);
                            uint256 uu2 = u2 | (u2 << 64);
                            n2 += ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        {
                            uint256 vv2 = n1 | (n1 >> 64);
                            n2 += ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        t0 = n1 | n2;
                    }
                    {
                        uint256 n1 = t1 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        n1 += ((t3 & 0xffffffff_ffffffff_00000000_00000000) << 128) | ((t3 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                        {
                            uint256 u1 = ((t1 & 0xffffffff_ffffffff_00000000_00000000) << 64) | ((t1 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                            uint256 uu1 = u1 | (u1 << 64);
                            n1 += ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        {
                            uint256 v1 = t0 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            uint256 vv1 = v1 | (v1 << 64);
                            n1 += ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        uint256 n2 = t1 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        n2 += ((t3 & 0xffffffff_ffffffff) << 128) | (t0 >> 192);
                        {
                            uint256 u2 = ((t1 & 0xffffffff_ffffffff) << 128) | (t2 >> 192);
                            uint256 uu2 = u2 | (u2 << 64);
                            n2 += ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        {
                            uint256 vv2 = n1 | (n1 >> 64);
                            n2 += ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        t1 = n1 | n2;
                    }
                    {
                        uint256 n1 = t2 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        n1 += ((t0 & 0xffffffff_ffffffff_00000000_00000000) << 128) | ((t0 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                        {
                            uint256 u1 = ((t2 & 0xffffffff_ffffffff_00000000_00000000) << 64) | ((t2 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                            uint256 uu1 = u1 | (u1 << 64);
                            n1 += ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        {
                            uint256 v1 = t1 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            uint256 vv1 = v1 | (v1 << 64);
                            n1 += ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        uint256 n2 = t2 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        n2 += ((t0 & 0xffffffff_ffffffff) << 128) | (t1 >> 192);
                        {
                            uint256 u2 = ((t2 & 0xffffffff_ffffffff) << 128) | (t3 >> 192);
                            uint256 uu2 = u2 | (u2 << 64);
                            n2 += ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        {
                            uint256 vv2 = n1 | (n1 >> 64);
                            n2 += ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        t2 = n1 | n2;
                    }
                    {
                        uint256 n1 = t3 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        n1 += ((t1 & 0xffffffff_ffffffff_00000000_00000000) << 128) | ((t1 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                        {
                            uint256 u1 = ((t3 & 0xffffffff_ffffffff_00000000_00000000) << 64) | ((t3 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                            uint256 uu1 = u1 | (u1 << 64);
                            n1 += ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        {
                            uint256 v1 = t2 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            uint256 vv1 = v1 | (v1 << 64);
                            n1 += ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        uint256 n2 = t3 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        n2 += ((t1 & 0xffffffff_ffffffff) << 128) | (t2 >> 192);
                        {
                            uint256 u2 = ((t3 & 0xffffffff_ffffffff) << 128) | (t0 >> 192);
                            uint256 uu2 = u2 | (u2 << 64);
                            n2 += ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        {
                            uint256 vv2 = n1 | (n1 >> 64);
                            n2 += ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        t3 = n1 | n2;
                    }
                    w3 = t3;
                    w2 = t2;
                }
                w1 = t1;
                w0 = t0;
            }
            uint256 h0 = ((a + 0x6a09e667_f3bcc908) & 0xffffffff_ffffffff) | (((b + 0xbb67ae85_84caa73b) & 0xffffffff_ffffffff) << 64) | (((c + 0x3c6ef372_fe94f82b) & 0xffffffff_ffffffff) << 128) | ((d + 0xa54ff53a_5f1d36f1) << 192);
            h0 = ((h0 & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((h0 & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8);
            h0 = ((h0 & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((h0 & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16);
            h0 = ((h0 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((h0 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32);
            uint256 h1 = ((e + 0x510e527f_ade682d1) & 0xffffffff_ffffffff) | (((f + 0x9b05688c_2b3e6c1f) & 0xffffffff_ffffffff) << 64) | (((g + 0x1f83d9ab_fb41bd6b) & 0xffffffff_ffffffff) << 128) | ((h + 0x5be0cd19_137e2179) << 192);
            h1 = ((h1 & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((h1 & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8);
            h1 = ((h1 & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((h1 & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16);
            h1 = ((h1 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((h1 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32);
            hh = addmod(h0, mulmod(h1, 0xfffffff_ffffffff_ffffffff_fffffffe_c6ef5bf4_737dcf70_d6ec3174_8d98951d, 0x10000000_00000000_00000000_00000000_14def9de_a2f79cd6_5812631a_5cf5d3ed), 0x10000000_00000000_00000000_00000000_14def9de_a2f79cd6_5812631a_5cf5d3ed);
        }
        // Step 2: unpack k
        k = bytes32(((uint256(k) & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((uint256(k) & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8));
        k = bytes32(((uint256(k) & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((uint256(k) & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16));
        k = bytes32(((uint256(k) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((uint256(k) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32));
        k = bytes32(((uint256(k) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff) << 64) | ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000) >> 64));
        k = bytes32((uint256(k) << 128) | (uint256(k) >> 128));
        uint256 ky = uint256(k) & 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff;
        uint256 kx;
        {
            uint256 ky2 = mulmod(ky, ky, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 u = addmod(ky2, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffec, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 v = mulmod(ky2, 0x52036cee_2b6ffe73_8cc74079_7779e898_00700a4d_4141d8ab_75eb4dca_135978a3, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed) + 1;
            uint256 t = mulmod(u, v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            (kx, ) = pow22501(t);
            kx = mulmod(kx, kx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            kx = mulmod(u, mulmod(mulmod(kx, kx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), t, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            t = mulmod(mulmod(kx, kx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            if (t != u) {
                if (t != 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - u) {
                    return false;
                }
                kx = mulmod(kx, 0x2b832480_4fc1df0b_2b4d0099_3dfbd7a7_2f431806_ad2fe478_c4ee1b27_4a0ea0b0, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
        }
        if ((kx & 1) != uint256(k) >> 255) {
            kx = 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - kx;
        }
        // Verify s
        s = bytes32(((uint256(s) & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((uint256(s) & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8));
        s = bytes32(((uint256(s) & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((uint256(s) & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16));
        s = bytes32(((uint256(s) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((uint256(s) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32));
        s = bytes32(((uint256(s) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff) << 64) | ((uint256(s) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000) >> 64));
        s = bytes32((uint256(s) << 128) | (uint256(s) >> 128));
        if (uint256(s) >= 0x10000000_00000000_00000000_00000000_14def9de_a2f79cd6_5812631a_5cf5d3ed) {
            return false;
        }
        uint256 vx;
        uint256 vu;
        uint256 vy;
        uint256 vv;
        // Step 3: compute multiples of k
        uint256[8][3][2] memory tables;
        {
            uint256 ks = ky + kx;
            uint256 kd = ky + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - kx;
            uint256 k2dt = mulmod(mulmod(kx, ky, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), 0x2406d9dc_56dffce7_198e80f2_eef3d130_00e0149a_8283b156_ebd69b94_26b2f159, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 kky = ky;
            uint256 kkx = kx;
            uint256 kku = 1;
            uint256 kkv = 1;
            {
                uint256 xx = mulmod(kkx, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy = mulmod(kky, kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz = mulmod(kku, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kkx = xxyy + xxyy;
                kku = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                kky = xx2 + yy2;
                kkv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
            {
                uint256 xx = mulmod(kkx, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy = mulmod(kky, kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz = mulmod(kku, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kkx = xxyy + xxyy;
                kku = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                kky = xx2 + yy2;
                kkv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
            {
                uint256 xx = mulmod(kkx, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy = mulmod(kky, kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz = mulmod(kku, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kkx = xxyy + xxyy;
                kku = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                kky = xx2 + yy2;
                kkv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
            uint256 cprod = 1;
            uint256[8][3][2] memory tables_ = tables;
            for (uint256 i = 0;; i++) {
                uint256 cs;
                uint256 cd;
                uint256 ct;
                uint256 c2z;
                {
                    uint256 cx = mulmod(kkx, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 cy = mulmod(kky, kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 cz = mulmod(kku, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    ct = mulmod(kkx, kky, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    cs = cy + cx;
                    cd = cy - cx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    c2z = cz + cz;
                }
                tables_[1][0][i] = cs;
                tables_[1][1][i] = cd;
                tables_[1][2][i] = mulmod(ct, 0x2406d9dc_56dffce7_198e80f2_eef3d130_00e0149a_8283b156_ebd69b94_26b2f159, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                tables_[0][0][i] = c2z;
                tables_[0][1][i] = cprod;
                cprod = mulmod(cprod, c2z, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                if (i == 7) {
                    break;
                }
                uint256 ab = mulmod(cs, ks, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 aa = mulmod(cd, kd, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 ac = mulmod(ct, k2dt, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kkx = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                kku = addmod(c2z, ac, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kky = ab + aa;
                kkv = addmod(c2z, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - ac, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
            uint256 t;
            (cprod, t) = pow22501(cprod);
        cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            cprod = mulmod(cprod, t, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            for (uint256 i = 7;; i--) {
                uint256 cinv = mulmod(cprod, tables_[0][1][i], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                tables_[1][0][i] = mulmod(tables_[1][0][i], cinv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                tables_[1][1][i] = mulmod(tables_[1][1][i], cinv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                tables_[1][2][i] = mulmod(tables_[1][2][i], cinv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                if (i == 0) {
                    break;
                }
                cprod = mulmod(cprod, tables_[0][0][i], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
            tables_[0] = [[0x43e7ce9d_19ea5d32_9385a44c_321ea161_67c996e3_7dc6070c_97de49e3_7ac61db9, 0x40cff344_25d8ec30_a3bb74ba_58cd5854_fa1e3818_6ad0d31e_bc8ae251_ceb2c97e, 0x459bd270_46e8dd45_aea7008d_b87a5a8f_79067792_53d64523_58951859_9fdfbf4b, 0x69fdd1e2_8c23cc38_94d0c8ff_90e76f6d_5b6e4c2e_620136d0_4dd83c4a_51581ab9, 0x54dceb34_13ce5cfa_11196dfc_960b6eda_f4b380c6_d4d23784_19cc0279_ba49c5f3, 0x4e24184d_d71a3d77_eef3729f_7f8cf7c1_7224cf40_aa7b9548_b9942f3c_5084ceed, 0x5a0e5aab_20262674_ae117576_1cbf5e88_9b52a55f_d7ac5027_c228cebd_c8d2360a, 0x26239334_073e9b38_c6285955_6d451c3d_cc8d30e8_4b361174_f488eadd_e2cf17d9], [0x227e97c9_4c7c0933_d2e0c21a_3447c504_fe9ccf82_e8a05f59_ce881c82_eba0489f, 0x226a3e0e_cc4afec6_fd0d2884_13014a9d_bddecf06_c1a2f0bb_702ba77c_613d8209, 0x34d7efc8_51d45c5e_71efeb0f_235b7946_91de6228_877569b3_a8d52bf0_58b8a4a0, 0x3c1f5fb3_ca7166fc_e1471c9b_752b6d28_c56301ad_7b65e845_1b2c8c55_26726e12, 0x6102416c_f02f02ff_5be75275_f55f28db_89b2a9d2_456b860c_e22fc0e5_031f7cc5, 0x40adf677_f1bfdae0_57f0fd17_9c126179_18ddaa28_91a6530f_b1a4294f_a8665490, 0x61936f3c_41560904_6187b8ba_a978cbc9_b4789336_3ae5a3cc_7d909f36_35ae7f48, 0x562a9662_b6ec47f9_e979d473_c02b51e4_42336823_8c58ddb5_2f0e5c6a_180e6410], [0x3788bdb4_4f8632d4_2d0dbee5_eea1acc6_136cf411_e655624f_55e48902_c3bd5534, 0x6190cf2c_2a7b5ad7_69d594a8_2844f23b_4167fa7c_8ac30e51_aa6cfbeb_dcd4b945, 0x65f77870_96be9204_123a71f3_ac88a87b_e1513217_737d6a1e_2f3a13a4_3d7e3a9a, 0x23af32d_bfa67975_536479a7_a7ce74a0_2142147f_ac048018_7f1f1334_9cda1f2d, 0x64fc44b7_fc6841bd_db0ced8b_8b0fe675_9137ef87_ee966512_15fc1dbc_d25c64dc, 0x1434aa37_48b701d5_b69df3d7_d340c1fe_3f6b9c1e_fc617484_caadb47e_382f4475, 0x457a6da8_c962ef35_f2b21742_3e5844e9_d2353452_7e8ea429_0d24e3dd_f21720c6, 0x63b9540c_eb60ccb5_1e4d989d_956e053c_f2511837_efb79089_d2ff4028_4202c53d]];
        }
        // Step 4: compute s*G - h*A
        {
            uint256 ss = uint256(s) << 3;
            uint256 hhh = hh + 0x80000000_00000000_00000000_00000000_a6f7cef5_17bce6b2_c09318d2_e7ae9f60;
            uint256 vvx = 0;
            uint256 vvu = 1;
            uint256 vvy = 1;
            uint256 vvv = 1;
            for (uint256 i = 252;; i--) {
                uint256 bit = 8 << i;
                if ((ss & bit) != 0) {
                    uint256 ws;
                    uint256 wd;
                    uint256 wz;
                    uint256 wt;
                    {
                        uint256 wx = mulmod(vvx, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        uint256 wy = mulmod(vvy, vvu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        ws = wy + wx;
                        wd = wy - wx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        wz = mulmod(vvu, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        wt = mulmod(vvx, vvy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    }
                    uint256 j = (ss >> i) & 7;
                    ss &= ~(7 << i);
                    uint256[8][3][2] memory tables_ = tables;
                    uint256 aa = mulmod(wd, tables_[0][1][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ab = mulmod(ws, tables_[0][0][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ac = mulmod(wt, tables_[0][2][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    vvx = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    vvu = wz + ac;
                    vvy = ab + aa;
                    vvv = wz - ac + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                }
                if ((hhh & bit) != 0) {
                    uint256 ws;
                    uint256 wd;
                    uint256 wz;
                    uint256 wt;
                    {
                        uint256 wx = mulmod(vvx, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        uint256 wy = mulmod(vvy, vvu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        ws = wy + wx;
                        wd = wy - wx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        wz = mulmod(vvu, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        wt = mulmod(vvx, vvy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    }
                    uint256 j = (hhh >> i) & 7;
                    hhh &= ~(7 << i);
                    uint256[8][3][2] memory tables_ = tables;
                    uint256 aa = mulmod(wd, tables_[1][0][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ab = mulmod(ws, tables_[1][1][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ac = mulmod(wt, tables_[1][2][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    vvx = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    vvu = wz - ac + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    vvy = ab + aa;
                    vvv = wz + ac;
                }
                if (i == 0) {
                    uint256 ws;
                    uint256 wd;
                    uint256 wz;
                    uint256 wt;
                    {
                        uint256 wx = mulmod(vvx, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        uint256 wy = mulmod(vvy, vvu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        ws = wy + wx;
                        wd = wy - wx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        wz = mulmod(vvu, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        wt = mulmod(vvx, vvy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    }
                    uint256 j = hhh & 7;
                    uint256[8][3][2] memory tables_ = tables;
                    uint256 aa = mulmod(wd, tables_[1][0][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ab = mulmod(ws, tables_[1][1][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ac = mulmod(wt, tables_[1][2][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    vvx = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    vvu = wz - ac + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    vvy = ab + aa;
                    vvv = wz + ac;
                    break;
                }
                {
                    uint256 xx = mulmod(vvx, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 yy = mulmod(vvy, vvu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 zz = mulmod(vvu, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    vvx = xxyy + xxyy;
                    vvu = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    vvy = xx2 + yy2;
                    vvv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - vvu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                }
            }
            vx = vvx;
            vu = vvu;
            vy = vvy;
            vv = vvv;
        }
        // Step 5: compare the points
        (uint256 vi, uint256 vj) = pow22501(mulmod(vu, vv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed));
        vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        vi = mulmod(vi, vj, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        vx = mulmod(vx, mulmod(vi, vv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        vy = mulmod(vy, mulmod(vi, vu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        bytes32 v = bytes32(vy | (vx << 255));
        v = bytes32(((uint256(v) & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((uint256(v) & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8));
        v = bytes32(((uint256(v) & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((uint256(v) & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16));
        v = bytes32(((uint256(v) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((uint256(v) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32));
        v = bytes32(((uint256(v) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff) << 64) | ((uint256(v) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000) >> 64));
        v = bytes32((uint256(v) << 128) | (uint256(v) >> 128));
        return v == r;
    }
}

// File: contracts/NearBridge.sol

pragma solidity ^0.5.0;
pragma experimental ABIEncoderV2; // solium-disable-line no-experimental







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
