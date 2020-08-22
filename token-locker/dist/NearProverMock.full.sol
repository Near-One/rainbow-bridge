
// File: ../nearprover/contracts/INearProver.sol

pragma solidity ^0.5.0;

interface INearProver {
    function proveOutcome(bytes calldata proofData, uint64 blockHeight) external view returns(bool);
}

// File: contracts/NearProverMock.sol

pragma solidity ^0.5.0;



contract NearProverMock is INearProver {
    function proveOutcome(bytes memory proofData, uint64 blockHeight) public view returns(bool) {
        return true;
    }
}
