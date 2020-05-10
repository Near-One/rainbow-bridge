pragma solidity ^0.5.0;

import "@openzeppelin/contracts/math/SafeMath.sol";


library ED25519 {
    function verify(bytes32 hash, bytes32 publicKey, bytes32[2] memory signature) internal view returns(bool success) {
        bytes32[4] memory data = [
            hash,
            publicKey,
            signature[0],
            signature[1]
        ];

        bool success;
        uint32[1] memory result = [uint32(7)];
        // solium-disable-next-line security/no-inline-assembly
        assembly{
            success := staticcall(gas, 0x9, add(data, 32), 128, result, 4)
            switch success
                case 0 { revert(0, returndatasize) }
        }

        return result[0] == 0;
    }
}