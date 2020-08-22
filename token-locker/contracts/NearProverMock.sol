pragma solidity ^0.5.0;

import "../../nearprover/contracts/INearProver.sol";


contract NearProverMock is INearProver {
    function proveOutcome(bytes memory proofData, uint64 blockHeight) public view returns(bool) {
        return true;
    }
}