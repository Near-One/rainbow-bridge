pragma solidity ^0.5.0;
pragma experimental ABIEncoderV2; // solium-disable-line no-experimental

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/ownership/Ownable.sol";
import "../../nearbridge/contracts/INearBridge.sol";
import "../../nearbridge/contracts/NearDecoder.sol";
import "./ProofDecoder.sol";


contract NearProver {
    using SafeMath for uint256;
    using Borsh for Borsh.Data;
    using NearDecoder for Borsh.Data;
    using ProofDecoder for Borsh.Data;

    INearBridge public bridge;

    constructor(INearBridge _bridge) public {
        bridge = _bridge;
    }

    function proveOutcome(bytes memory proofData, bytes32 blockMerkleRoot) public view returns(bool) {
        Borsh.Data memory borshData = Borsh.from(proofData);
        ProofDecoder.FullOutcomeProof memory fullOutcomeProof = borshData.decodeFullOutcomeProof();
        require(borshData.finished(), "NearProver: argument should be exact borsh serialization");

        bytes32 hash = _computeRoot(
            fullOutcomeProof.outcome_proof.outcome_with_id.hash,
            fullOutcomeProof.outcome_proof.proof
        );

        hash = sha256(abi.encodePacked(hash));

        //TODO(Anton): please refactor this code out
        hash = _computeRoot(
            hash,
            fullOutcomeProof.outcome_root_proof
        );
//        revert("outcome root computation done");

        require(
            hash == fullOutcomeProof.block_header_lite.inner_lite.outcome_root,
            "NearProver: merkle proof is not valid"
        );

//        revert("before block proof computation");

        require(
            _computeRoot(fullOutcomeProof.block_header_lite.hash, fullOutcomeProof.block_proof) == blockMerkleRoot, "NearProver: block proof is not valid"
        );
//        revert("after block proof computation");
        return true;
    }

    function _computeRoot(bytes32 node, ProofDecoder.MerklePath memory proof) internal pure returns(bytes32 hash) {
        hash = node;
        for (uint i = 0; i < proof.items.length; i++) {
            ProofDecoder.MerklePathItem memory item = proof.items[i];
            if (item.direction == 0) {
                hash = sha256(abi.encodePacked(item.hash, hash));
            }
            else {
                hash = sha256(abi.encodePacked(hash, item.hash));
            }
        }
    }
}
