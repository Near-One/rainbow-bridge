// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity 0.8.7;

import "@openzeppelin/contracts-upgradeable/access/AccessControlUpgradeable.sol";

contract AdminControlled is AccessControlUpgradeable {
    uint public paused;

    bytes32 public constant PAUSE_ROLE = keccak256("PAUSE_ROLE");
    bytes32 public constant UPGRADER_ROLE = keccak256("BRIDGE_UPGRADER_ROLE");

    modifier pausable(uint flag) {
        require((paused & flag) == 0 || hasRole(DEFAULT_ADMIN_ROLE, _msgSender()), "Paused");
        _;
    }

    function __AdminControlled_init(uint _flags, address upgrader) public onlyInitializing {
        __AccessControl_init();
        paused = _flags;
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(PAUSE_ROLE, msg.sender);
        _grantRole(UPGRADER_ROLE, upgrader);
    }

    function transferUpgraderAdmin(address newUpgrader) public onlyRole(UPGRADER_ROLE) {
        require(newUpgrader != address(0), "new upgrader is the zero address");
        _grantRole(UPGRADER_ROLE, newUpgrader);
        _revokeRole(UPGRADER_ROLE, _msgSender());
        emit UpgraderOwnershipTransferred(_msgSender(), newUpgrader);

    }

    function adminPause(uint flags) external onlyRole(PAUSE_ROLE) {
        paused = flags;
    }

    function transferOwnership(address newAdmin) external virtual onlyRole(DEFAULT_ADMIN_ROLE) {
        require(newAdmin != address(0), "Ownable: new owner is the zero address");
        _grantRole(DEFAULT_ADMIN_ROLE, newAdmin);
        _grantRole(PAUSE_ROLE, newAdmin);

        _revokeRole(PAUSE_ROLE, _msgSender());
        _revokeRole(DEFAULT_ADMIN_ROLE, _msgSender());
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
    event UpgraderOwnershipTransferred(address oldUpgrader, address newUpgrader);
}
