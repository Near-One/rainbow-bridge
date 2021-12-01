// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.7;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

contract AdminControlled is Initializable {
    address public admin;
    uint public paused;

    function __AdminControlled_init(address _admin, uint flags) public initializer {
        admin = _admin;
        paused = flags;
    }

    modifier onlyAdmin() {
        require(msg.sender == admin, "Unauthorized");
        _;
    }

    modifier pausable(uint flag) {
        require((paused & flag) == 0 || msg.sender == admin);
        _;
    }

    function adminPause(uint flags) external onlyAdmin {
        paused = flags;
    }

    function transferOwnership(address newAdmin) external virtual onlyAdmin {
        require(newAdmin != address(0), "Ownable: new owner is the zero address");
        address oldAdmin = admin;
        admin = newAdmin;
        emit OwnershipTransferred(oldAdmin, newAdmin);
    }

    function adminSstore(uint key, uint value) external onlyAdmin {
        assembly {
            sstore(key, value)
        }
    }

    function adminSstoreWithMask(
        uint key,
        uint value,
        uint mask
    ) external onlyAdmin {
        assembly {
            let oldval := sload(key)
            sstore(key, xor(and(xor(value, oldval), mask), oldval))
        }
    }

    function adminSendEth(address payable destination, uint amount) external onlyAdmin {
        destination.transfer(amount);
    }

    function adminReceiveEth() external payable onlyAdmin {}

    event OwnershipTransferred(address oldAdmin, address newAdmin);
}
