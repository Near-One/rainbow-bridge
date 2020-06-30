
// File: contracts/INearProver.sol

pragma solidity ^0.5.0;

interface INearProver {
    function proveOutcome(bytes calldata proofData, uint256 blockHeight) external view returns(bool);
}
