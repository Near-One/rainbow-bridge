// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8;

import "../NearProver.sol";

contract NearProverV2 is NearProver {
    function version() pure external returns (string memory) {
        return "2.0.0";
    }
}
