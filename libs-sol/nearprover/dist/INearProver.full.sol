
// File: contracts/INearProver.sol

pragma solidity ^0.5.0;

interface INearProver {
    function proveOutcome(bytes calldata proofData, uint64 blockHeight) external view returns(bool);
}
