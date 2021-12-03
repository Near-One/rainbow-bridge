// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.7;

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";

contract AdminControlled is AccessControlUpgradeable {
    uint public paused;
    address public admin;

    bytes32 public constant PAUSE_ROLE = keccak256("NEAR_BRIDGE_PAUSE");

    modifier pausable(uint flag) {
        require((paused & flag) == 0 || hasRole(DEFAULT_ADMIN_ROLE, _msgSender()));
        _;
    }

    function __AdminControlled_init(uint flags) public initializer {
        __AccessControl_init();

        _setupRole(DEFAULT_ADMIN_ROLE, _msgSender());
        _setupRole(PAUSE_ROLE, _msgSender());
        paused = flags;
    }

    function adminPause(uint flags) external onlyRole(PAUSE_ROLE) {
        paused = flags;
    }

    function transferOwnership(address newAdmin) external virtual onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newAdmin != address(0), "Ownable: new owner is the zero address");
        _grantRole(DEFAULT_ADMIN_ROLE, newAdmin);
        _grantRole(PAUSE_ROLE, newAdmin);
        admin = newAdmin;

        revokeRole(PAUSE_ROLE, _msgSender());
        revokeRole(DEFAULT_ADMIN_ROLE, _msgSender());
        emit OwnershipTransferred(_msgSender(), newAdmin);
    }

    function adminSstore(uint key, uint value) external onlyRole(DEFAULT_ADMIN_ROLE) {
        assembly {
            sstore(key, value)
        }
    }

    function adminSstoreWithMask(
        uint key,
        uint value,
        uint mask
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        assembly {
            let oldval := sload(key)
            sstore(key, xor(and(xor(value, oldval), mask), oldval))
        }
    }

    function adminSendEth(address payable destination, uint amount) external onlyRole(DEFAULT_ADMIN_ROLE) {
        destination.transfer(amount);
    }

    function adminReceiveEth() external payable {}

    event OwnershipTransferred(address oldAdmin, address newAdmin);
}
