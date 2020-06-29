pragma solidity ^0.5.0;
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "../../nearprover/contracts/INearProver.sol";
import "../../nearprover/contracts/ProofDecoder.sol";
import "../../nearbridge/contracts/NearDecoder.sol";
import "../../nearbridge/contracts/Borsh.sol";

contract TokenLocker {
    using SafeERC20 for IERC20;
    using Borsh for Borsh.Data;
    using ProofDecoder for Borsh.Data;
    using NearDecoder for Borsh.Data;

    IERC20 public ethToken_;
    bytes public nearToken_;
    INearProver public prover_;

    mapping(bytes32 => bool) public usedEvents_;

    event Locked(
        address indexed token,
        address indexed sender,
        uint256 amount,
        string accountId
    );

    event Unlocked(
        uint128 amount,
        address recipient
    );

    // Function output from burning fungible token on Near side.
    struct BurnResult {
        uint128 amount;
        address recipient;
    }

    function _decodeBurnResult(bytes memory data) internal pure returns(BurnResult memory result) {
        Borsh.Data memory borshData = Borsh.from(data);
        result.amount = borshData.decodeU128();
        bytes20 recipient = borshData.decodeBytes20();
        result.recipient = address(uint160(recipient));
    }

    function lockToken(uint256 amount, string memory accountId) public {
        ethToken_.safeTransferFrom(msg.sender, address(this), amount);
        emit Locked(address(ethToken_), msg.sender, amount, accountId);
    }

    function uintToString(uint256 v) internal pure returns (string memory str) {
        uint maxlength = 100;
        bytes memory reversed = new bytes(maxlength);
        uint i = 0;
        while (v != 0) {
            uint remainder = v % 10;
            v = v / 10;
            reversed[i++] = bytes1(uint8(48 + remainder));
        }
        bytes memory s = new bytes(i + 1);
        for (uint j = 0; j <= i; j++) {
            s[j] = reversed[i - j];
        }
        str = string(s);
    }

    function char(byte b) internal pure returns (byte c) {
        if (uint8(b) < uint8(10)) return byte(uint8(b) + 0x30);
        else return byte(uint8(b) + 0x57);
    }

    function bytes32string(bytes32 b32) internal pure returns (string memory out) {
        bytes memory s = new bytes(64);

        for (uint i = 0; i < 32; i++) {
            byte b = byte(b32[i]);
            byte hi = byte(uint8(b) / 16);
            byte lo = byte(uint8(b) - 16 * uint8(hi));
            s[i*2] = char(hi);
            s[i*2+1] = char(lo);
        }

        out = string(s);
    }

    function unlockToken(bytes memory proofData, uint256 proofBlockHeight) public {
        bytes32 key = keccak256(proofData);
        require(!usedEvents_[key], "The burn event cannot be reused");
        usedEvents_[key] = true;
        require(prover_.proveOutcome(proofData, proofBlockHeight), "Proof should be valid");

        // Unpack the proof and extract the execution outcome.
        Borsh.Data memory borshData = Borsh.from(proofData);
        ProofDecoder.FullOutcomeProof memory fullOutcomeProof = borshData.decodeFullOutcomeProof();
        require(borshData.finished(), "Argument should be exact borsh serialization");

        require(keccak256(fullOutcomeProof.outcome_proof.outcome_with_id.outcome.executor_id) == keccak256(nearToken_),
        "Can only unlock tokens from the linked mintable fungible token on Near blockchain.");

        ProofDecoder.ExecutionStatus memory status = fullOutcomeProof.outcome_proof.outcome_with_id.outcome.status;
        require(!status.failed, "Cannot use failed execution outcome for unlocking the tokens.");
        require(!status.unknown, "Cannot use unknown execution outcome for unlocking the tokens.");
        BurnResult memory result = _decodeBurnResult(status.successValue);
//        revert(uintToString(uint256(result.amount)));
         ethToken_.transfer(result.recipient, result.amount);
        emit Unlocked(result.amount, result.recipient);
    }

    // TokenLocker is linked to the fungible token on Ethereum side and mintable fungible
    // token on NEAR side, it also links to the prover that it uses to unlock the tokens.
    constructor(IERC20 ethToken, bytes memory nearToken, INearProver prover) public {
        ethToken_ = ethToken;
        nearToken_ = nearToken;
        prover_ = prover;
    }
}
