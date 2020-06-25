pragma solidity ^0.5.0;
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/SafeERC20.sol";
import "../../nearprover/contracts/NearProver.sol";
import "../../nearprover/contracts/ProofDecoder.sol";
import "../../nearbridge/contracts/NearDecoder.sol";
import "../../nearbridge/contracts/Borsh.sol";

contract TokenLocker {
    using SafeERC20 for IERC20;
    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;
    using ProofDecoder for Borsh.Data;

    string public nearToken_;
    string public ethToken_;
    NearProver public prover_;

    mapping(bytes32 => bool) public usedEvents_;

    event Locked(
        address indexed token,
        address indexed sender,
        uint256 amount,
        string accountId
    );

    // Function output from burning fungible token on Near side.
    struct BurnResult {
        uint128 amount;
        address recipient;
    }

    function decodeBurnResult(Borsh.data memory data) internal pure returns(BurnResult memory result) {
        result.amount = data.decodeU128();
        bytes32 recipient = data.decodeBytes32();
        result.recipient = address(recipient);
    }

    function lockToken(uint256 amount, string memory accountId) public {
        nearToken_.safeTransferFrom(msg.sender, address(this), amount);
        emit Locked(address(nearToken_), msg.sender, amount, accountId);
    }

    function unlockToken(bytes memory proofData, uint256 proofBlockHeight) {
        bytes32 key = keccak256(proofData);
        require(!usedEvents_[key], "The burn event cannot be reused");
        usedEvents_[key] = true;
        require(prover_.proveOutcome(proofData, blockHeight), "Proof should be valid");

        // Unpack the proof and extract the execution outcome.
        Borsh.Data memory borshData = Borsh.from(proofData);
        ProofDecoder.FullOutcomeProof memory fullOutcomeProof = borshData.decodeFullOutcomeProof();
        require(borshData.finished(), "NearProver: argument should be exact borsh serialization");

        // TODO: Check that correct fun token burnt the amount.
        ProofDecoder.ExecutionStatus memory status = fullOutcomeProof.outcome_proof.outcome_with_id.outcome.status;
        require(!status.failed, "Cannot use failed execution outcome for unlocking the tokens.");
        require(!status.unknown, "Cannot use unknown execution outcome for unlocking the tokens.");
        Borsh.Data memory valueData = Borsh.from(status.successValue);

        BurnResult memory result = valueData.decodeBurnResult();
    }

    // TokenLocker is linked to the fungible token on Ethereum side and mintable fungible
    // token on NEAR side, it also links to the prover that it uses to unlock the tokens.
    constructor(IERC20 ethToken, string memory nearToken, NearProver prover) public {
        nearToken_ = nearToken;
        ethToken_ = ethToken;
        prover_ = prover;
    }
}
