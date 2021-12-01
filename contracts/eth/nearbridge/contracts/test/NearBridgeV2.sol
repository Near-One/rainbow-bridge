// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8;

import "../NearBridge.sol";

contract NearBridgeV2 is NearBridge {
    function version() pure external returns (string memory) {
        return "2.0.0";
    }
}
