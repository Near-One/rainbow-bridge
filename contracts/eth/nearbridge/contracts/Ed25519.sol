// SPDX-License-Identifier: GPL-3.0-or-later
pragma solidity ^0.8;

contract Ed25519 {
    // Computes (v^(2^250-1), v^11) mod p
    function pow22501(uint256 v) private pure returns (uint256 p22501, uint256 p11) {
        p11 = mulmod(v, v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(p11, p11, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(
            mulmod(p22501, p22501, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed),
            v,
            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
        );
        p11 = mulmod(p22501, p11, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(
            mulmod(p11, p11, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed),
            p22501,
            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
        );
        uint256 a = mulmod(p22501, p22501, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(p22501, p22501, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        uint256 b = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(p22501, p22501, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        b = mulmod(b, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, b, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        a = mulmod(a, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        p22501 = mulmod(p22501, a, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
    }

    function check(
        bytes32 k,
        bytes32 r,
        bytes32 s,
        bytes32 m1,
        bytes9 m2
    ) public pure returns (bool) {
        unchecked {
            uint256 hh;
            // Step 1: compute SHA-512(R, A, M)
            {
                uint256[5][16] memory kk = [
                    [
                        uint256(0x428a2f98_d728ae22),
                        uint256(0xe49b69c1_9ef14ad2),
                        uint256(0x27b70a85_46d22ffc),
                        uint256(0x19a4c116_b8d2d0c8),
                        uint256(0xca273ece_ea26619c)
                    ],
                    [
                        uint256(0x71374491_23ef65cd),
                        uint256(0xefbe4786_384f25e3),
                        uint256(0x2e1b2138_5c26c926),
                        uint256(0x1e376c08_5141ab53),
                        uint256(0xd186b8c7_21c0c207)
                    ],
                    [
                        uint256(0xb5c0fbcf_ec4d3b2f),
                        uint256(0xfc19dc6_8b8cd5b5),
                        uint256(0x4d2c6dfc_5ac42aed),
                        uint256(0x2748774c_df8eeb99),
                        uint256(0xeada7dd6_cde0eb1e)
                    ],
                    [
                        uint256(0xe9b5dba5_8189dbbc),
                        uint256(0x240ca1cc_77ac9c65),
                        uint256(0x53380d13_9d95b3df),
                        uint256(0x34b0bcb5_e19b48a8),
                        uint256(0xf57d4f7f_ee6ed178)
                    ],
                    [
                        uint256(0x3956c25b_f348b538),
                        uint256(0x2de92c6f_592b0275),
                        uint256(0x650a7354_8baf63de),
                        uint256(0x391c0cb3_c5c95a63),
                        uint256(0x6f067aa_72176fba)
                    ],
                    [
                        uint256(0x59f111f1_b605d019),
                        uint256(0x4a7484aa_6ea6e483),
                        uint256(0x766a0abb_3c77b2a8),
                        uint256(0x4ed8aa4a_e3418acb),
                        uint256(0xa637dc5_a2c898a6)
                    ],
                    [
                        uint256(0x923f82a4_af194f9b),
                        uint256(0x5cb0a9dc_bd41fbd4),
                        uint256(0x81c2c92e_47edaee6),
                        uint256(0x5b9cca4f_7763e373),
                        uint256(0x113f9804_bef90dae)
                    ],
                    [
                        uint256(0xab1c5ed5_da6d8118),
                        uint256(0x76f988da_831153b5),
                        uint256(0x92722c85_1482353b),
                        uint256(0x682e6ff3_d6b2b8a3),
                        uint256(0x1b710b35_131c471b)
                    ],
                    [
                        uint256(0xd807aa98_a3030242),
                        uint256(0x983e5152_ee66dfab),
                        uint256(0xa2bfe8a1_4cf10364),
                        uint256(0x748f82ee_5defb2fc),
                        uint256(0x28db77f5_23047d84)
                    ],
                    [
                        uint256(0x12835b01_45706fbe),
                        uint256(0xa831c66d_2db43210),
                        uint256(0xa81a664b_bc423001),
                        uint256(0x78a5636f_43172f60),
                        uint256(0x32caab7b_40c72493)
                    ],
                    [
                        uint256(0x243185be_4ee4b28c),
                        uint256(0xb00327c8_98fb213f),
                        uint256(0xc24b8b70_d0f89791),
                        uint256(0x84c87814_a1f0ab72),
                        uint256(0x3c9ebe0a_15c9bebc)
                    ],
                    [
                        uint256(0x550c7dc3_d5ffb4e2),
                        uint256(0xbf597fc7_beef0ee4),
                        uint256(0xc76c51a3_0654be30),
                        uint256(0x8cc70208_1a6439ec),
                        uint256(0x431d67c4_9c100d4c)
                    ],
                    [
                        uint256(0x72be5d74_f27b896f),
                        uint256(0xc6e00bf3_3da88fc2),
                        uint256(0xd192e819_d6ef5218),
                        uint256(0x90befffa_23631e28),
                        uint256(0x4cc5d4be_cb3e42b6)
                    ],
                    [
                        uint256(0x80deb1fe_3b1696b1),
                        uint256(0xd5a79147_930aa725),
                        uint256(0xd6990624_5565a910),
                        uint256(0xa4506ceb_de82bde9),
                        uint256(0x597f299c_fc657e2a)
                    ],
                    [
                        uint256(0x9bdc06a7_25c71235),
                        uint256(0x6ca6351_e003826f),
                        uint256(0xf40e3585_5771202a),
                        uint256(0xbef9a3f7_b2c67915),
                        uint256(0x5fcb6fab_3ad6faec)
                    ],
                    [
                        uint256(0xc19bf174_cf692694),
                        uint256(0x14292967_0a0e6e70),
                        uint256(0x106aa070_32bbd1b8),
                        uint256(0xc67178f2_e372532b),
                        uint256(0x6c44198c_4a475817)
                    ]
                ];
                uint256 w0 = (uint256(r) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_ffffffff_ffffffff) |
                    ((uint256(r) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) |
                    ((uint256(r) & 0xffffffff_ffffffff_00000000_00000000) << 64);
                uint256 w1 = (uint256(k) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_ffffffff_ffffffff) |
                    ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) |
                    ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000) << 64);
                uint256 w2 = (uint256(m1) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_ffffffff_ffffffff) |
                    ((uint256(m1) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) |
                    ((uint256(m1) & 0xffffffff_ffffffff_00000000_00000000) << 64);
                uint256 w3 = (uint256(bytes32(m2)) &
                    0xffffffff_ffffffff_00000000_00000000_00000000_00000000_00000000_00000000) |
                    ((uint256(bytes32(m2)) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) |
                    0x800000_00000000_00000000_00000348;
                uint256 a = 0x6a09e667_f3bcc908;
                uint256 b = 0xbb67ae85_84caa73b;
                uint256 c = 0x3c6ef372_fe94f82b;
                uint256 d = 0xa54ff53a_5f1d36f1;
                uint256 e = 0x510e527f_ade682d1;
                uint256 f = 0x9b05688c_2b3e6c1f;
                uint256 g = 0x1f83d9ab_fb41bd6b;
                uint256 h = 0x5be0cd19_137e2179;
                for (uint256 i = 0; ; i++) {
                    // Round 16 * i
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[0][i];
                        temp1 += w0 >> 192;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 1
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[1][i];
                        temp1 += w0 >> 64;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 2
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[2][i];
                        temp1 += w0 >> 128;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 3
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[3][i];
                        temp1 += w0;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 4
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[4][i];
                        temp1 += w1 >> 192;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 5
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[5][i];
                        temp1 += w1 >> 64;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 6
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[6][i];
                        temp1 += w1 >> 128;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 7
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[7][i];
                        temp1 += w1;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 8
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[8][i];
                        temp1 += w2 >> 192;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 9
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[9][i];
                        temp1 += w2 >> 64;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 10
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[10][i];
                        temp1 += w2 >> 128;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 11
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[11][i];
                        temp1 += w2;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 12
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[12][i];
                        temp1 += w3 >> 192;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 13
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[13][i];
                        temp1 += w3 >> 64;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 14
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[14][i];
                        temp1 += w3 >> 128;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    // Round 16 * i + 15
                    {
                        uint256 temp1;
                        uint256 temp2;
                        e &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = e | (e << 64);
                            uint256 s1 = (ss >> 14) ^ (ss >> 18) ^ (ss >> 41);
                            uint256 ch = (e & (f ^ g)) ^ g;
                            temp1 = h + s1 + ch;
                        }
                        temp1 += kk[15][i];
                        temp1 += w3;
                        a &= 0xffffffff_ffffffff;
                        {
                            uint256 ss = a | (a << 64);
                            uint256 s0 = (ss >> 28) ^ (ss >> 34) ^ (ss >> 39);
                            uint256 maj = (a & (b | c)) | (b & c);
                            temp2 = s0 + maj;
                        }
                        h = g;
                        g = f;
                        f = e;
                        e = d + temp1;
                        d = c;
                        c = b;
                        b = a;
                        a = temp1 + temp2;
                    }
                    if (i == 4) {
                        break;
                    }
                    // Message expansion
                    uint256 t0 = w0;
                    uint256 t1 = w1;
                    {
                        uint256 t2 = w2;
                        uint256 t3 = w3;
                        {
                            uint256 n1 = t0 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            n1 +=
                                ((t2 & 0xffffffff_ffffffff_00000000_00000000) << 128) |
                                ((t2 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                            {
                                uint256 u1 = ((t0 & 0xffffffff_ffffffff_00000000_00000000) << 64) |
                                    ((t0 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                                uint256 uu1 = u1 | (u1 << 64);
                                n1 +=
                                    ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            }
                            {
                                uint256 v1 = t3 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                                uint256 vv1 = v1 | (v1 << 64);
                                n1 +=
                                    ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            }
                            n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            uint256 n2 = t0 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            n2 += ((t2 & 0xffffffff_ffffffff) << 128) | (t3 >> 192);
                            {
                                uint256 u2 = ((t0 & 0xffffffff_ffffffff) << 128) | (t1 >> 192);
                                uint256 uu2 = u2 | (u2 << 64);
                                n2 +=
                                    ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            }
                            {
                                uint256 vv2 = n1 | (n1 >> 64);
                                n2 +=
                                    ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            }
                            n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            t0 = n1 | n2;
                        }
                        {
                            uint256 n1 = t1 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            n1 +=
                                ((t3 & 0xffffffff_ffffffff_00000000_00000000) << 128) |
                                ((t3 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                            {
                                uint256 u1 = ((t1 & 0xffffffff_ffffffff_00000000_00000000) << 64) |
                                    ((t1 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                                uint256 uu1 = u1 | (u1 << 64);
                                n1 +=
                                    ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            }
                            {
                                uint256 v1 = t0 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                                uint256 vv1 = v1 | (v1 << 64);
                                n1 +=
                                    ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            }
                            n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            uint256 n2 = t1 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            n2 += ((t3 & 0xffffffff_ffffffff) << 128) | (t0 >> 192);
                            {
                                uint256 u2 = ((t1 & 0xffffffff_ffffffff) << 128) | (t2 >> 192);
                                uint256 uu2 = u2 | (u2 << 64);
                                n2 +=
                                    ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            }
                            {
                                uint256 vv2 = n1 | (n1 >> 64);
                                n2 +=
                                    ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            }
                            n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            t1 = n1 | n2;
                        }
                        {
                            uint256 n1 = t2 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            n1 +=
                                ((t0 & 0xffffffff_ffffffff_00000000_00000000) << 128) |
                                ((t0 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                            {
                                uint256 u1 = ((t2 & 0xffffffff_ffffffff_00000000_00000000) << 64) |
                                    ((t2 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                                uint256 uu1 = u1 | (u1 << 64);
                                n1 +=
                                    ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            }
                            {
                                uint256 v1 = t1 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                                uint256 vv1 = v1 | (v1 << 64);
                                n1 +=
                                    ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            }
                            n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            uint256 n2 = t2 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            n2 += ((t0 & 0xffffffff_ffffffff) << 128) | (t1 >> 192);
                            {
                                uint256 u2 = ((t2 & 0xffffffff_ffffffff) << 128) | (t3 >> 192);
                                uint256 uu2 = u2 | (u2 << 64);
                                n2 +=
                                    ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            }
                            {
                                uint256 vv2 = n1 | (n1 >> 64);
                                n2 +=
                                    ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            }
                            n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            t2 = n1 | n2;
                        }
                        {
                            uint256 n1 = t3 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            n1 +=
                                ((t1 & 0xffffffff_ffffffff_00000000_00000000) << 128) |
                                ((t1 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                            {
                                uint256 u1 = ((t3 & 0xffffffff_ffffffff_00000000_00000000) << 64) |
                                    ((t3 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                                uint256 uu1 = u1 | (u1 << 64);
                                n1 +=
                                    ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            }
                            {
                                uint256 v1 = t2 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                                uint256 vv1 = v1 | (v1 << 64);
                                n1 +=
                                    ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            }
                            n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                            uint256 n2 = t3 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            n2 += ((t1 & 0xffffffff_ffffffff) << 128) | (t2 >> 192);
                            {
                                uint256 u2 = ((t3 & 0xffffffff_ffffffff) << 128) | (t0 >> 192);
                                uint256 uu2 = u2 | (u2 << 64);
                                n2 +=
                                    ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            }
                            {
                                uint256 vv2 = n1 | (n1 >> 64);
                                n2 +=
                                    ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) &
                                    0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            }
                            n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            t3 = n1 | n2;
                        }
                        w3 = t3;
                        w2 = t2;
                    }
                    w1 = t1;
                    w0 = t0;
                }
                uint256 h0 = ((a + 0x6a09e667_f3bcc908) & 0xffffffff_ffffffff) |
                    (((b + 0xbb67ae85_84caa73b) & 0xffffffff_ffffffff) << 64) |
                    (((c + 0x3c6ef372_fe94f82b) & 0xffffffff_ffffffff) << 128) |
                    ((d + 0xa54ff53a_5f1d36f1) << 192);
                h0 =
                    ((h0 & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) |
                    ((h0 & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8);
                h0 =
                    ((h0 & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) |
                    ((h0 & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16);
                h0 =
                    ((h0 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) |
                    ((h0 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32);
                uint256 h1 = ((e + 0x510e527f_ade682d1) & 0xffffffff_ffffffff) |
                    (((f + 0x9b05688c_2b3e6c1f) & 0xffffffff_ffffffff) << 64) |
                    (((g + 0x1f83d9ab_fb41bd6b) & 0xffffffff_ffffffff) << 128) |
                    ((h + 0x5be0cd19_137e2179) << 192);
                h1 =
                    ((h1 & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) |
                    ((h1 & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8);
                h1 =
                    ((h1 & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) |
                    ((h1 & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16);
                h1 =
                    ((h1 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) |
                    ((h1 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32);
                hh = addmod(
                    h0,
                    mulmod(
                        h1,
                        0xfffffff_ffffffff_ffffffff_fffffffe_c6ef5bf4_737dcf70_d6ec3174_8d98951d,
                        0x10000000_00000000_00000000_00000000_14def9de_a2f79cd6_5812631a_5cf5d3ed
                    ),
                    0x10000000_00000000_00000000_00000000_14def9de_a2f79cd6_5812631a_5cf5d3ed
                );
            }
            // Step 2: unpack k
            k = bytes32(
                ((uint256(k) & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) |
                    ((uint256(k) & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8)
            );
            k = bytes32(
                ((uint256(k) & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) |
                    ((uint256(k) & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16)
            );
            k = bytes32(
                ((uint256(k) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) |
                    ((uint256(k) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32)
            );
            k = bytes32(
                ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff) << 64) |
                    ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000) >> 64)
            );
            k = bytes32((uint256(k) << 128) | (uint256(k) >> 128));
            uint256 ky = uint256(k) & 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff;
            uint256 kx;
            {
                uint256 ky2 = mulmod(ky, ky, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 u = addmod(
                    ky2,
                    0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffec,
                    0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                );
                uint256 v = mulmod(
                    ky2,
                    0x52036cee_2b6ffe73_8cc74079_7779e898_00700a4d_4141d8ab_75eb4dca_135978a3,
                    0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                ) + 1;
                uint256 t = mulmod(u, v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                (kx, ) = pow22501(t);
                kx = mulmod(kx, kx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kx = mulmod(
                    u,
                    mulmod(
                        mulmod(kx, kx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed),
                        t,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    ),
                    0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                );
                t = mulmod(
                    mulmod(kx, kx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed),
                    v,
                    0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                );
                if (t != u) {
                    if (t != 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - u) {
                        return false;
                    }
                    kx = mulmod(
                        kx,
                        0x2b832480_4fc1df0b_2b4d0099_3dfbd7a7_2f431806_ad2fe478_c4ee1b27_4a0ea0b0,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                }
            }
            if ((kx & 1) != uint256(k) >> 255) {
                kx = 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - kx;
            }
            // Verify s
            s = bytes32(
                ((uint256(s) & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) |
                    ((uint256(s) & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8)
            );
            s = bytes32(
                ((uint256(s) & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) |
                    ((uint256(s) & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16)
            );
            s = bytes32(
                ((uint256(s) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) |
                    ((uint256(s) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32)
            );
            s = bytes32(
                ((uint256(s) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff) << 64) |
                    ((uint256(s) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000) >> 64)
            );
            s = bytes32((uint256(s) << 128) | (uint256(s) >> 128));
            if (uint256(s) >= 0x10000000_00000000_00000000_00000000_14def9de_a2f79cd6_5812631a_5cf5d3ed) {
                return false;
            }
            uint256 vx;
            uint256 vu;
            uint256 vy;
            uint256 vv;
            // Step 3: compute multiples of k
            uint256[8][3][2] memory tables;
            {
                uint256 ks = ky + kx;
                uint256 kd = ky + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - kx;
                uint256 k2dt = mulmod(
                    mulmod(kx, ky, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed),
                    0x2406d9dc_56dffce7_198e80f2_eef3d130_00e0149a_8283b156_ebd69b94_26b2f159,
                    0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                );
                uint256 kky = ky;
                uint256 kkx = kx;
                uint256 kku = 1;
                uint256 kkv = 1;
                {
                    uint256 xx = mulmod(
                        kkx,
                        kkv,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 yy = mulmod(
                        kky,
                        kku,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 zz = mulmod(
                        kku,
                        kkv,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 xx2 = mulmod(
                        xx,
                        xx,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 yy2 = mulmod(
                        yy,
                        yy,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 xxyy = mulmod(
                        xx,
                        yy,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 zz2 = mulmod(
                        zz,
                        zz,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    kkx = xxyy + xxyy;
                    kku = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    kky = xx2 + yy2;
                    kkv = addmod(
                        zz2 + zz2,
                        0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - kku,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                }
                {
                    uint256 xx = mulmod(
                        kkx,
                        kkv,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 yy = mulmod(
                        kky,
                        kku,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 zz = mulmod(
                        kku,
                        kkv,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 xx2 = mulmod(
                        xx,
                        xx,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 yy2 = mulmod(
                        yy,
                        yy,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 xxyy = mulmod(
                        xx,
                        yy,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 zz2 = mulmod(
                        zz,
                        zz,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    kkx = xxyy + xxyy;
                    kku = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    kky = xx2 + yy2;
                    kkv = addmod(
                        zz2 + zz2,
                        0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - kku,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                }
                {
                    uint256 xx = mulmod(
                        kkx,
                        kkv,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 yy = mulmod(
                        kky,
                        kku,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 zz = mulmod(
                        kku,
                        kkv,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 xx2 = mulmod(
                        xx,
                        xx,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 yy2 = mulmod(
                        yy,
                        yy,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 xxyy = mulmod(
                        xx,
                        yy,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 zz2 = mulmod(
                        zz,
                        zz,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    kkx = xxyy + xxyy;
                    kku = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    kky = xx2 + yy2;
                    kkv = addmod(
                        zz2 + zz2,
                        0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - kku,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                }
                uint256 cprod = 1;
                uint256[8][3][2] memory tables_ = tables;
                for (uint256 i = 0; ; i++) {
                    uint256 cs;
                    uint256 cd;
                    uint256 ct;
                    uint256 c2z;
                    {
                        uint256 cx = mulmod(
                            kkx,
                            kkv,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 cy = mulmod(
                            kky,
                            kku,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 cz = mulmod(
                            kku,
                            kkv,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        ct = mulmod(
                            kkx,
                            kky,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        cs = cy + cx;
                        cd = cy - cx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        c2z = cz + cz;
                    }
                    tables_[1][0][i] = cs;
                    tables_[1][1][i] = cd;
                    tables_[1][2][i] = mulmod(
                        ct,
                        0x2406d9dc_56dffce7_198e80f2_eef3d130_00e0149a_8283b156_ebd69b94_26b2f159,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    tables_[0][0][i] = c2z;
                    tables_[0][1][i] = cprod;
                    cprod = mulmod(
                        cprod,
                        c2z,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    if (i == 7) {
                        break;
                    }
                    uint256 ab = mulmod(
                        cs,
                        ks,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 aa = mulmod(
                        cd,
                        kd,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    uint256 ac = mulmod(
                        ct,
                        k2dt,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    kkx = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    kku = addmod(c2z, ac, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    kky = ab + aa;
                    kkv = addmod(
                        c2z,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - ac,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                }
                uint256 t;
                (cprod, t) = pow22501(cprod);
                cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                cprod = mulmod(cprod, cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                cprod = mulmod(cprod, t, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                for (uint256 i = 7; ; i--) {
                    uint256 cinv = mulmod(
                        cprod,
                        tables_[0][1][i],
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    tables_[1][0][i] = mulmod(
                        tables_[1][0][i],
                        cinv,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    tables_[1][1][i] = mulmod(
                        tables_[1][1][i],
                        cinv,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    tables_[1][2][i] = mulmod(
                        tables_[1][2][i],
                        cinv,
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                    if (i == 0) {
                        break;
                    }
                    cprod = mulmod(
                        cprod,
                        tables_[0][0][i],
                        0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                    );
                }
                tables_[0] = [
                    [
                        0x43e7ce9d_19ea5d32_9385a44c_321ea161_67c996e3_7dc6070c_97de49e3_7ac61db9,
                        0x40cff344_25d8ec30_a3bb74ba_58cd5854_fa1e3818_6ad0d31e_bc8ae251_ceb2c97e,
                        0x459bd270_46e8dd45_aea7008d_b87a5a8f_79067792_53d64523_58951859_9fdfbf4b,
                        0x69fdd1e2_8c23cc38_94d0c8ff_90e76f6d_5b6e4c2e_620136d0_4dd83c4a_51581ab9,
                        0x54dceb34_13ce5cfa_11196dfc_960b6eda_f4b380c6_d4d23784_19cc0279_ba49c5f3,
                        0x4e24184d_d71a3d77_eef3729f_7f8cf7c1_7224cf40_aa7b9548_b9942f3c_5084ceed,
                        0x5a0e5aab_20262674_ae117576_1cbf5e88_9b52a55f_d7ac5027_c228cebd_c8d2360a,
                        0x26239334_073e9b38_c6285955_6d451c3d_cc8d30e8_4b361174_f488eadd_e2cf17d9
                    ],
                    [
                        0x227e97c9_4c7c0933_d2e0c21a_3447c504_fe9ccf82_e8a05f59_ce881c82_eba0489f,
                        0x226a3e0e_cc4afec6_fd0d2884_13014a9d_bddecf06_c1a2f0bb_702ba77c_613d8209,
                        0x34d7efc8_51d45c5e_71efeb0f_235b7946_91de6228_877569b3_a8d52bf0_58b8a4a0,
                        0x3c1f5fb3_ca7166fc_e1471c9b_752b6d28_c56301ad_7b65e845_1b2c8c55_26726e12,
                        0x6102416c_f02f02ff_5be75275_f55f28db_89b2a9d2_456b860c_e22fc0e5_031f7cc5,
                        0x40adf677_f1bfdae0_57f0fd17_9c126179_18ddaa28_91a6530f_b1a4294f_a8665490,
                        0x61936f3c_41560904_6187b8ba_a978cbc9_b4789336_3ae5a3cc_7d909f36_35ae7f48,
                        0x562a9662_b6ec47f9_e979d473_c02b51e4_42336823_8c58ddb5_2f0e5c6a_180e6410
                    ],
                    [
                        0x3788bdb4_4f8632d4_2d0dbee5_eea1acc6_136cf411_e655624f_55e48902_c3bd5534,
                        0x6190cf2c_2a7b5ad7_69d594a8_2844f23b_4167fa7c_8ac30e51_aa6cfbeb_dcd4b945,
                        0x65f77870_96be9204_123a71f3_ac88a87b_e1513217_737d6a1e_2f3a13a4_3d7e3a9a,
                        0x23af32d_bfa67975_536479a7_a7ce74a0_2142147f_ac048018_7f1f1334_9cda1f2d,
                        0x64fc44b7_fc6841bd_db0ced8b_8b0fe675_9137ef87_ee966512_15fc1dbc_d25c64dc,
                        0x1434aa37_48b701d5_b69df3d7_d340c1fe_3f6b9c1e_fc617484_caadb47e_382f4475,
                        0x457a6da8_c962ef35_f2b21742_3e5844e9_d2353452_7e8ea429_0d24e3dd_f21720c6,
                        0x63b9540c_eb60ccb5_1e4d989d_956e053c_f2511837_efb79089_d2ff4028_4202c53d
                    ]
                ];
            }
            // Step 4: compute s*G - h*A
            {
                uint256 ss = uint256(s) << 3;
                uint256 hhh = hh + 0x80000000_00000000_00000000_00000000_a6f7cef5_17bce6b2_c09318d2_e7ae9f60;
                uint256 vvx = 0;
                uint256 vvu = 1;
                uint256 vvy = 1;
                uint256 vvv = 1;
                for (uint256 i = 252; ; i--) {
                    uint256 bit = 8 << i;
                    if ((ss & bit) != 0) {
                        uint256 ws;
                        uint256 wd;
                        uint256 wz;
                        uint256 wt;
                        {
                            uint256 wx = mulmod(
                                vvx,
                                vvv,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                            uint256 wy = mulmod(
                                vvy,
                                vvu,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                            ws = wy + wx;
                            wd = wy - wx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                            wz = mulmod(
                                vvu,
                                vvv,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                            wt = mulmod(
                                vvx,
                                vvy,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                        }
                        uint256 j = (ss >> i) & 7;
                        ss &= ~(7 << i);
                        uint256[8][3][2] memory tables_ = tables;
                        uint256 aa = mulmod(
                            wd,
                            tables_[0][1][j],
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 ab = mulmod(
                            ws,
                            tables_[0][0][j],
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 ac = mulmod(
                            wt,
                            tables_[0][2][j],
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        vvx = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        vvu = wz + ac;
                        vvy = ab + aa;
                        vvv = wz - ac + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    }
                    if ((hhh & bit) != 0) {
                        uint256 ws;
                        uint256 wd;
                        uint256 wz;
                        uint256 wt;
                        {
                            uint256 wx = mulmod(
                                vvx,
                                vvv,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                            uint256 wy = mulmod(
                                vvy,
                                vvu,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                            ws = wy + wx;
                            wd = wy - wx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                            wz = mulmod(
                                vvu,
                                vvv,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                            wt = mulmod(
                                vvx,
                                vvy,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                        }
                        uint256 j = (hhh >> i) & 7;
                        hhh &= ~(7 << i);
                        uint256[8][3][2] memory tables_ = tables;
                        uint256 aa = mulmod(
                            wd,
                            tables_[1][0][j],
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 ab = mulmod(
                            ws,
                            tables_[1][1][j],
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 ac = mulmod(
                            wt,
                            tables_[1][2][j],
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        vvx = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        vvu = wz - ac + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        vvy = ab + aa;
                        vvv = wz + ac;
                    }
                    if (i == 0) {
                        uint256 ws;
                        uint256 wd;
                        uint256 wz;
                        uint256 wt;
                        {
                            uint256 wx = mulmod(
                                vvx,
                                vvv,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                            uint256 wy = mulmod(
                                vvy,
                                vvu,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                            ws = wy + wx;
                            wd = wy - wx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                            wz = mulmod(
                                vvu,
                                vvv,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                            wt = mulmod(
                                vvx,
                                vvy,
                                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                            );
                        }
                        uint256 j = hhh & 7;
                        uint256[8][3][2] memory tables_ = tables;
                        uint256 aa = mulmod(
                            wd,
                            tables_[1][0][j],
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 ab = mulmod(
                            ws,
                            tables_[1][1][j],
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 ac = mulmod(
                            wt,
                            tables_[1][2][j],
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        vvx = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        vvu = wz - ac + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        vvy = ab + aa;
                        vvv = wz + ac;
                        break;
                    }
                    {
                        uint256 xx = mulmod(
                            vvx,
                            vvv,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 yy = mulmod(
                            vvy,
                            vvu,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 zz = mulmod(
                            vvu,
                            vvv,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 xx2 = mulmod(
                            xx,
                            xx,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 yy2 = mulmod(
                            yy,
                            yy,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 xxyy = mulmod(
                            xx,
                            yy,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        uint256 zz2 = mulmod(
                            zz,
                            zz,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                        vvx = xxyy + xxyy;
                        vvu = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        vvy = xx2 + yy2;
                        vvv = addmod(
                            zz2 + zz2,
                            0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - vvu,
                            0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
                        );
                    }
                }
                vx = vvx;
                vu = vvu;
                vy = vvy;
                vv = vvv;
            }
            // Step 5: compare the points
            (uint256 vi, uint256 vj) = pow22501(
                mulmod(vu, vv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed)
            );
            vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            vi = mulmod(vi, vi, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            vi = mulmod(vi, vj, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            vx = mulmod(
                vx,
                mulmod(vi, vv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed),
                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
            );
            vy = mulmod(
                vy,
                mulmod(vi, vu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed),
                0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed
            );
            bytes32 vr = bytes32(vy | (vx << 255));
            vr = bytes32(
                ((uint256(vr) & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) |
                    ((uint256(vr) & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8)
            );
            vr = bytes32(
                ((uint256(vr) & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) |
                    ((uint256(vr) & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16)
            );
            vr = bytes32(
                ((uint256(vr) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) |
                    ((uint256(vr) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32)
            );
            vr = bytes32(
                ((uint256(vr) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff) << 64) |
                    ((uint256(vr) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000) >> 64)
            );
            vr = bytes32((uint256(vr) << 128) | (uint256(vr) >> 128));
            return vr == r;
        }
    }
}
