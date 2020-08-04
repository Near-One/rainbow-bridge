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

    // OutcomeReciptId -> Used
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

    function unlockToken(bytes memory proofData, uint64 proofBlockHeight) public {
        require(prover_.proveOutcome(proofData, proofBlockHeight), "Proof should be valid");

        // Unpack the proof and extract the execution outcome.
        Borsh.Data memory borshData = Borsh.from(proofData);
        ProofDecoder.FullOutcomeProof memory fullOutcomeProof = borshData.decodeFullOutcomeProof();
        require(borshData.finished(), "Argument should be exact borsh serialization");

        bytes32 receiptId = fullOutcomeProof.outcome_proof.outcome_with_id.outcome.receipt_ids[0];
        require(!usedEvents_[receiptId], "The burn event cannot be reused");
        usedEvents_[receiptId] = true;

        require(keccak256(fullOutcomeProof.outcome_proof.outcome_with_id.outcome.executor_id) == keccak256(nearToken_),
        "Can only unlock tokens from the linked mintable fungible token on Near blockchain.");

        ProofDecoder.ExecutionStatus memory status = fullOutcomeProof.outcome_proof.outcome_with_id.outcome.status;
        require(!status.failed, "Cannot use failed execution outcome for unlocking the tokens.");
        require(!status.unknown, "Cannot use unknown execution outcome for unlocking the tokens.");
        BurnResult memory result = _decodeBurnResult(status.successValue);
        ethToken_.safeTransfer(result.recipient, result.amount);
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
