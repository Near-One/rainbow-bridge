pragma solidity >=0.5;


// https://gist.github.com/abacabadabacaba/cb927b5ebe6db63f00b98902503ffc76
// https://gist.github.com/abacabadabacaba/9c395588c455ca1f7dccfa853d8fd56d
// https://play.golang.org/p/obqA-XDYzdm
library Ed25519 {
    function expmod(uint256 x, uint256 y, uint256 k) private view returns (uint256 r) {
        assembly {
            let m := mload(0x40)
            mstore(m, 32)
            mstore(add(m, 0x20), 32)
            mstore(add(m, 0x40), 32)
            mstore(add(m, 0x60), x)
            mstore(add(m, 0x80), y)
            mstore(add(m, 0xa0), k)
            if iszero(staticcall(gas(), 5, m, 0xc0, 0, 0x20)) {
                invalid()
            }
            r := mload(0)
        }
    }

    function check(bytes32 k, bytes32 r, bytes32 s, bytes32 m1, bytes9 m2) internal view returns (bool) {
        uint256 hh;
        // Step 1: compute SHA-512(R, A, M)
        {
            uint256[5][16] memory kk = [[uint256(0x428a2f98_d728ae22), uint256(0xe49b69c1_9ef14ad2), uint256(0x27b70a85_46d22ffc), uint256(0x19a4c116_b8d2d0c8), uint256(0xca273ece_ea26619c)], [uint256(0x71374491_23ef65cd), uint256(0xefbe4786_384f25e3), uint256(0x2e1b2138_5c26c926), uint256(0x1e376c08_5141ab53), uint256(0xd186b8c7_21c0c207)], [uint256(0xb5c0fbcf_ec4d3b2f), uint256(0xfc19dc6_8b8cd5b5), uint256(0x4d2c6dfc_5ac42aed), uint256(0x2748774c_df8eeb99), uint256(0xeada7dd6_cde0eb1e)], [uint256(0xe9b5dba5_8189dbbc), uint256(0x240ca1cc_77ac9c65), uint256(0x53380d13_9d95b3df), uint256(0x34b0bcb5_e19b48a8), uint256(0xf57d4f7f_ee6ed178)], [uint256(0x3956c25b_f348b538), uint256(0x2de92c6f_592b0275), uint256(0x650a7354_8baf63de), uint256(0x391c0cb3_c5c95a63), uint256(0x6f067aa_72176fba)], [uint256(0x59f111f1_b605d019), uint256(0x4a7484aa_6ea6e483), uint256(0x766a0abb_3c77b2a8), uint256(0x4ed8aa4a_e3418acb), uint256(0xa637dc5_a2c898a6)], [uint256(0x923f82a4_af194f9b), uint256(0x5cb0a9dc_bd41fbd4), uint256(0x81c2c92e_47edaee6), uint256(0x5b9cca4f_7763e373), uint256(0x113f9804_bef90dae)], [uint256(0xab1c5ed5_da6d8118), uint256(0x76f988da_831153b5), uint256(0x92722c85_1482353b), uint256(0x682e6ff3_d6b2b8a3), uint256(0x1b710b35_131c471b)], [uint256(0xd807aa98_a3030242), uint256(0x983e5152_ee66dfab), uint256(0xa2bfe8a1_4cf10364), uint256(0x748f82ee_5defb2fc), uint256(0x28db77f5_23047d84)], [uint256(0x12835b01_45706fbe), uint256(0xa831c66d_2db43210), uint256(0xa81a664b_bc423001), uint256(0x78a5636f_43172f60), uint256(0x32caab7b_40c72493)], [uint256(0x243185be_4ee4b28c), uint256(0xb00327c8_98fb213f), uint256(0xc24b8b70_d0f89791), uint256(0x84c87814_a1f0ab72), uint256(0x3c9ebe0a_15c9bebc)], [uint256(0x550c7dc3_d5ffb4e2), uint256(0xbf597fc7_beef0ee4), uint256(0xc76c51a3_0654be30), uint256(0x8cc70208_1a6439ec), uint256(0x431d67c4_9c100d4c)], [uint256(0x72be5d74_f27b896f), uint256(0xc6e00bf3_3da88fc2), uint256(0xd192e819_d6ef5218), uint256(0x90befffa_23631e28), uint256(0x4cc5d4be_cb3e42b6)], [uint256(0x80deb1fe_3b1696b1), uint256(0xd5a79147_930aa725), uint256(0xd6990624_5565a910), uint256(0xa4506ceb_de82bde9), uint256(0x597f299c_fc657e2a)], [uint256(0x9bdc06a7_25c71235), uint256(0x6ca6351_e003826f), uint256(0xf40e3585_5771202a), uint256(0xbef9a3f7_b2c67915), uint256(0x5fcb6fab_3ad6faec)], [uint256(0xc19bf174_cf692694), uint256(0x14292967_0a0e6e70), uint256(0x106aa070_32bbd1b8), uint256(0xc67178f2_e372532b), uint256(0x6c44198c_4a475817)]];
            uint256 w0 = (uint256(r) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_ffffffff_ffffffff) | ((uint256(r) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) | ((uint256(r) & 0xffffffff_ffffffff_00000000_00000000) << 64);
            uint256 w1 = (uint256(k) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_ffffffff_ffffffff) | ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) | ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000) << 64);
            uint256 w2 = (uint256(m1) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_ffffffff_ffffffff) | ((uint256(m1) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) | ((uint256(m1) & 0xffffffff_ffffffff_00000000_00000000) << 64);
            uint256 w3 = (uint256(bytes32(m2)) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000_00000000_00000000) | ((uint256(bytes32(m2)) & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64) | 0x800000_00000000_00000000_00000348;
            uint256 a = 0x6a09e667_f3bcc908;
            uint256 b = 0xbb67ae85_84caa73b;
            uint256 c = 0x3c6ef372_fe94f82b;
            uint256 d = 0xa54ff53a_5f1d36f1;
            uint256 e = 0x510e527f_ade682d1;
            uint256 f = 0x9b05688c_2b3e6c1f;
            uint256 g = 0x1f83d9ab_fb41bd6b;
            uint256 h = 0x5be0cd19_137e2179;
            for (uint256 i = 0;; i++) {
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
                        n1 += ((t2 & 0xffffffff_ffffffff_00000000_00000000) << 128) | ((t2 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                        {
                            uint256 u1 = ((t0 & 0xffffffff_ffffffff_00000000_00000000) << 64) | ((t0 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                            uint256 uu1 = u1 | (u1 << 64);
                            n1 += ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        {
                            uint256 v1 = t3 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            uint256 vv1 = v1 | (v1 << 64);
                            n1 += ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        uint256 n2 = t0 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        n2 += ((t2 & 0xffffffff_ffffffff) << 128) | (t3 >> 192);
                        {
                            uint256 u2 = ((t0 & 0xffffffff_ffffffff) << 128) | (t1 >> 192);
                            uint256 uu2 = u2 | (u2 << 64);
                            n2 += ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        {
                            uint256 vv2 = n1 | (n1 >> 64);
                            n2 += ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        t0 = n1 | n2;
                    }
                    {
                        uint256 n1 = t1 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        n1 += ((t3 & 0xffffffff_ffffffff_00000000_00000000) << 128) | ((t3 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                        {
                            uint256 u1 = ((t1 & 0xffffffff_ffffffff_00000000_00000000) << 64) | ((t1 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                            uint256 uu1 = u1 | (u1 << 64);
                            n1 += ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        {
                            uint256 v1 = t0 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            uint256 vv1 = v1 | (v1 << 64);
                            n1 += ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        uint256 n2 = t1 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        n2 += ((t3 & 0xffffffff_ffffffff) << 128) | (t0 >> 192);
                        {
                            uint256 u2 = ((t1 & 0xffffffff_ffffffff) << 128) | (t2 >> 192);
                            uint256 uu2 = u2 | (u2 << 64);
                            n2 += ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        {
                            uint256 vv2 = n1 | (n1 >> 64);
                            n2 += ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        t1 = n1 | n2;
                    }
                    {
                        uint256 n1 = t2 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        n1 += ((t0 & 0xffffffff_ffffffff_00000000_00000000) << 128) | ((t0 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                        {
                            uint256 u1 = ((t2 & 0xffffffff_ffffffff_00000000_00000000) << 64) | ((t2 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                            uint256 uu1 = u1 | (u1 << 64);
                            n1 += ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        {
                            uint256 v1 = t1 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            uint256 vv1 = v1 | (v1 << 64);
                            n1 += ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        uint256 n2 = t2 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        n2 += ((t0 & 0xffffffff_ffffffff) << 128) | (t1 >> 192);
                        {
                            uint256 u2 = ((t2 & 0xffffffff_ffffffff) << 128) | (t3 >> 192);
                            uint256 uu2 = u2 | (u2 << 64);
                            n2 += ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        {
                            uint256 vv2 = n1 | (n1 >> 64);
                            n2 += ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        n2 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        t2 = n1 | n2;
                    }
                    {
                        uint256 n1 = t3 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        n1 += ((t1 & 0xffffffff_ffffffff_00000000_00000000) << 128) | ((t1 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 64);
                        {
                            uint256 u1 = ((t3 & 0xffffffff_ffffffff_00000000_00000000) << 64) | ((t3 & 0xffffffff_ffffffff_00000000_00000000_00000000_00000000) >> 128);
                            uint256 uu1 = u1 | (u1 << 64);
                            n1 += ((uu1 << 63) ^ (uu1 << 56) ^ (u1 << 57)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        {
                            uint256 v1 = t2 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                            uint256 vv1 = v1 | (v1 << 64);
                            n1 += ((vv1 << 45) ^ (vv1 << 3) ^ (v1 << 58)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        }
                        n1 &= 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000;
                        uint256 n2 = t3 & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        n2 += ((t1 & 0xffffffff_ffffffff) << 128) | (t2 >> 192);
                        {
                            uint256 u2 = ((t3 & 0xffffffff_ffffffff) << 128) | (t0 >> 192);
                            uint256 uu2 = u2 | (u2 << 64);
                            n2 += ((uu2 >> 1) ^ (uu2 >> 8) ^ (u2 >> 7)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
                        }
                        {
                            uint256 vv2 = n1 | (n1 >> 64);
                            n2 += ((vv2 >> 19) ^ (vv2 >> 61) ^ (n1 >> 70)) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff;
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
            uint256 h0 = ((a + 0x6a09e667_f3bcc908) & 0xffffffff_ffffffff) | (((b + 0xbb67ae85_84caa73b) & 0xffffffff_ffffffff) << 64) | (((c + 0x3c6ef372_fe94f82b) & 0xffffffff_ffffffff) << 128) | ((d + 0xa54ff53a_5f1d36f1) << 192);
            h0 = ((h0 & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((h0 & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8);
            h0 = ((h0 & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((h0 & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16);
            h0 = ((h0 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((h0 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32);
            uint256 h1 = ((e + 0x510e527f_ade682d1) & 0xffffffff_ffffffff) | (((f + 0x9b05688c_2b3e6c1f) & 0xffffffff_ffffffff) << 64) | (((g + 0x1f83d9ab_fb41bd6b) & 0xffffffff_ffffffff) << 128) | ((h + 0x5be0cd19_137e2179) << 192);
            h1 = ((h1 & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((h1 & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8);
            h1 = ((h1 & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((h1 & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16);
            h1 = ((h1 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((h1 & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32);
            hh = addmod(h0, mulmod(h1, 0xfffffff_ffffffff_ffffffff_fffffffe_c6ef5bf4_737dcf70_d6ec3174_8d98951d, 0x10000000_00000000_00000000_00000000_14def9de_a2f79cd6_5812631a_5cf5d3ed), 0x10000000_00000000_00000000_00000000_14def9de_a2f79cd6_5812631a_5cf5d3ed);
        }
        // Step 2: unpack and verify the points
        k = bytes32(((uint256(k) & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((uint256(k) & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8));
        k = bytes32(((uint256(k) & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((uint256(k) & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16));
        k = bytes32(((uint256(k) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((uint256(k) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32));
        k = bytes32(((uint256(k) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff) << 64) | ((uint256(k) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000) >> 64));
        k = bytes32((uint256(k) << 128) | (uint256(k) >> 128));
        uint256 ky = uint256(k) & 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff;
        if (ky >= 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed) {
            return false;
        }
        uint256 kx;
        {
            uint256 ky2 = mulmod(ky, ky, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 u = addmod(ky2, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffec, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 v = mulmod(ky2, 0x52036cee_2b6ffe73_8cc74079_7779e898_00700a4d_4141d8ab_75eb4dca_135978a3, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed) + 1;
            kx = mulmod(u, expmod(mulmod(u, v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), 0xfffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_fffffffd, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 t = mulmod(mulmod(kx, kx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            if (t != u) {
                if (t != 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - u) {
                    return false;
                }
                kx = mulmod(kx, 0x2b832480_4fc1df0b_2b4d0099_3dfbd7a7_2f431806_ad2fe478_c4ee1b27_4a0ea0b0, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
        }
        if ((kx & 1) != uint256(k) >> 255) {
            if (kx == 0) {
                return false;
            }
            kx = 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - kx;
        }
        r = bytes32(((uint256(r) & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((uint256(r) & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8));
        r = bytes32(((uint256(r) & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((uint256(r) & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16));
        r = bytes32(((uint256(r) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((uint256(r) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32));
        r = bytes32(((uint256(r) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff) << 64) | ((uint256(r) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000) >> 64));
        r = bytes32((uint256(r) << 128) | (uint256(r) >> 128));
        uint256 ry = uint256(r) & 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff;
        if (ry >= 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed) {
            return false;
        }
        uint256 rx;
        {
            uint256 ry2 = mulmod(ry, ry, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 u = addmod(ry2, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffec, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 v = mulmod(ry2, 0x52036cee_2b6ffe73_8cc74079_7779e898_00700a4d_4141d8ab_75eb4dca_135978a3, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed) + 1;
            rx = mulmod(u, expmod(mulmod(u, v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), 0xfffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_fffffffd, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 t = mulmod(mulmod(rx, rx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed), v, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            if (t != u) {
                if (t != 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - u) {
                    return false;
                }
                rx = mulmod(rx, 0x2b832480_4fc1df0b_2b4d0099_3dfbd7a7_2f431806_ad2fe478_c4ee1b27_4a0ea0b0, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
        }
        if ((rx & 1) != uint256(r) >> 255) {
            if (rx == 0) {
                return false;
            }
            rx = 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - rx;
        }
        // Verify the scalar as well
        s = bytes32(((uint256(s) & 0xff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff_00ff00ff) << 8) | ((uint256(s) & 0xff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00_ff00ff00) >> 8));
        s = bytes32(((uint256(s) & 0xffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff_0000ffff) << 16) | ((uint256(s) & 0xffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000_ffff0000) >> 16));
        s = bytes32(((uint256(s) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff) << 32) | ((uint256(s) & 0xffffffff_00000000_ffffffff_00000000_ffffffff_00000000_ffffffff_00000000) >> 32));
        s = bytes32(((uint256(s) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff) << 64) | ((uint256(s) & 0xffffffff_ffffffff_00000000_00000000_ffffffff_ffffffff_00000000_00000000) >> 64));
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
            uint256 kt = mulmod(kx, ky, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 kky = ky;
            uint256 kkx = kx;
            uint256 kku = 1;
            uint256 kkv = 1;
            {
                uint256 xx = mulmod(kkx, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy = mulmod(kky, kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz = mulmod(kku, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kkx = xxyy + xxyy;
                kku = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                kky = xx2 + yy2;
                kkv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
            {
                uint256 xx = mulmod(kkx, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy = mulmod(kky, kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz = mulmod(kku, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kkx = xxyy + xxyy;
                kku = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                kky = xx2 + yy2;
                kkv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
            {
                uint256 xx = mulmod(kkx, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy = mulmod(kky, kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz = mulmod(kku, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kkx = xxyy + xxyy;
                kku = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                kky = xx2 + yy2;
                kkv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
            uint256 cprod = 1;
            uint256[8][3][2] memory tables_ = tables;
            for (uint256 i = 0;; i++) {
                uint256 cs;
                uint256 cd;
                uint256 c2t;
                uint256 c2z;
                {
                    uint256 cx = mulmod(kkx, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 cy = mulmod(kky, kku, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 cz = mulmod(kku, kkv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ct = mulmod(kkx, kky, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    cs = cy + cx;
                    cd = cy - cx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    c2t = ct + ct;
                    c2z = cz + cz;
                }
                tables_[1][0][i] = cs;
                tables_[1][1][i] = cd;
                tables_[1][2][i] = c2t;
                tables_[0][0][i] = c2z;
                tables_[0][1][i] = cprod;
                cprod = mulmod(cprod, c2z, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                if (i == 7) {
                    break;
                }
                uint256 aa = mulmod(cd, ks, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 ab = mulmod(cs, kd, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                uint256 ac = mulmod(c2z, kt, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kkx = addmod(c2t, ac, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kku = ab + aa;
                kky = addmod(c2t, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed - ac, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                kkv = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
            }
            cprod = expmod(cprod, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffeb, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            for (uint256 i = 7;; i--) {
                uint256 cinv = mulmod(cprod, tables_[0][1][i], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                tables_[1][0][i] = mulmod(tables_[1][0][i], cinv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                tables_[1][1][i] = mulmod(tables_[1][1][i], cinv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                tables_[1][2][i] = mulmod(tables_[1][2][i], cinv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                if (i == 0) {
                    break;
                }
                cprod = mulmod(cprod, tables_[0][0][i], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            }
            tables_[0] = [[0x448af3b0_30de7297_d899ce33_2f6ce191_b65981bb_f1443816_2cdbacb3_026e9f3e, 0x5a5cf699_c56ebfac_e75919e4_e3437ada_d3284f37_a8de2362_cd9733c5_5354318e, 0x21d63b14_5572c8f6_86aa81b1_cdaaa323_22fa9a6a_0b0bf02b_ff8ec9e9_59b03a47, 0x213ad571_2a36c7d7_889f4238_8bb81a03_72ecff67_81181713_97df8042_4540156f, 0x43c557ef_469e6eac_22f66e97_380d37c9_c46f1eeb_ec1a68d4_f11ac021_538f3a93, 0x51a7ebf7_61a37920_a9bd0709_7d83dd03_dfc259cd_5ade6f6d_dfb86111_51003fad, 0x3757b059_23268b18_0d23a602_1440deea_c013bfe0_2bf7d4c3_937c3876_1f909ef9, 0x30f1148b_f896f395_96dedefd_60f96a68_43246146_8c4e1236_92676601_89e7f550], [0x5d481250_990700e1_9613a0d6_371e11fd_cb1ebb40_70daac7c_f153aef6_f9c91a63, 0x24e02d28_fd6e4e47_cb65e304_73af5822_4c5040db_7a9020cd_f9b710bf_01cec032, 0x41a9c196_24a85b81_4239cba2_faee1cac_027c0c1b_32d4f817_baf9aac7_113840e5, 0xc55acc0_14eae3bf_d1d03aaa_b546f5ca_9f6b59b4_bb8441c0_98afad81_24c321a4, 0x7cab7671_45301176_c426fdb7_2b6aedee_91d95fc8_66592d92_42b7c1ba_da9eaa53, 0x4133c416_8bb01253_ce895919_5556acb4_577e75e4_cdbbb7b5_a837809d_993fdfc0, 0x16f87516_2d9e4069_08972b78_b77623d4_e734d907_edbe5198_efedc522_6a0b8940, 0x61dae6a1_0c682f5f_ba841804_03d928c9_69c14dd2_154c886b_0205e6c3_2346677c], [0x2c4f59ec_edf7eae1_1608c29b_38b3d993_45f0ce6d_344c2bf5_6fbef41a_d41a51bf, 0x5c70fc48_ea87cbf9_db6676ad_f747cd74_17b3d4a0_f770327e_5c3b386f_88b2f465, 0x36e05f32_66735292_88bfd103_09b4577f_74f5a9f4_b3b92525_43e74c95_035bc63e, 0x5ae6a565_800f28a2_39caa6be_f216adbf_590001dd_e18fc837_eb85cf2e_db5beed4, 0x2b344e20_3a4858a1_2067d3a8_31cd006f_0f1ef0c4_8c5b13ab_d5ecc312_8e4b1ccb, 0x412806b9_17be6460_c5c0dd61_cd562338_5b14aa51_eb2e8efc_522cdccd_e8de2f53, 0x66432d14_63a87e0f_8ea60abf_3cbfe479_71e7437f_e66445e0_89133d2c_a271c2e0, 0x5e33f00e_36b77491_ab24fd22_41760de2_fa63b68f_e40ae5f5_140d7f33_5c92bf29]];
        }
        // Step 4: compute 8*s*G - 8*h*A
        {
            uint256 ss = uint256(s) << 3;
            uint256 hhh = hh << 3;
            uint256 vvx = 0;
            uint256 vvu = 1;
            uint256 vvy = 1;
            uint256 vvv = 1;
            for (uint256 i = 252;; i--) {
                uint256 bit = 8 << i;
                if ((ss & bit) != 0) {
                    uint256 ws;
                    uint256 wd;
                    uint256 wz;
                    uint256 wt;
                    {
                        uint256 wx = mulmod(vvx, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        uint256 wy = mulmod(vvy, vvu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        ws = wy + wx;
                        wd = wy - wx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        wz = mulmod(vvu, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        wt = mulmod(vvx, vvy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    }
                    uint256 j = (ss >> i) & 7;
                    ss &= ~(7 << i);
                    uint256[8][3][2] memory tables_ = tables;
                    uint256 aa = mulmod(wd, tables_[0][0][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ab = mulmod(ws, tables_[0][1][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ac = mulmod(wz, tables_[0][2][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    vvx = wt + ac;
                    vvu = ab + aa;
                    vvy = wt - ac + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    vvv = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                }
                if ((hhh & bit) != 0) {
                    uint256 ws;
                    uint256 wd;
                    uint256 wz;
                    uint256 wt;
                    {
                        uint256 wx = mulmod(vvx, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        uint256 wy = mulmod(vvy, vvu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        ws = wy + wx;
                        wd = wy - wx + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                        wz = mulmod(vvu, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                        wt = mulmod(vvx, vvy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    }
                    uint256 j = (hhh >> i) & 7;
                    hhh &= ~(7 << i);
                    uint256[8][3][2] memory tables_ = tables;
                    uint256 aa = mulmod(wd, tables_[1][1][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ab = mulmod(ws, tables_[1][0][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 ac = mulmod(wz, tables_[1][2][j], 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    vvx = wt - ac + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    vvu = ab + aa;
                    vvy = wt + ac;
                    vvv = ab - aa + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                }
                if (i == 0) {
                    break;
                }
                {
                    uint256 xx = mulmod(vvx, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 yy = mulmod(vvy, vvu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 zz = mulmod(vvu, vvv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                    vvx = xxyy + xxyy;
                    vvu = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
                    vvy = xx2 + yy2;
                    vvv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - vvu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
                }
            }
            vx = vvx;
            vu = vvu;
            vy = vvy;
            vv = vvv;
        }
        // Step 5: compute 8*R
        uint256 ru = 1;
        uint256 rv = 1;
        {
            uint256 xx = mulmod(rx, rv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 yy = mulmod(ry, ru, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 zz = mulmod(ru, rv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            rx = xxyy + xxyy;
            ru = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
            ry = xx2 + yy2;
            rv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - ru, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        }
        {
            uint256 xx = mulmod(rx, rv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 yy = mulmod(ry, ru, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 zz = mulmod(ru, rv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            rx = xxyy + xxyy;
            ru = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
            ry = xx2 + yy2;
            rv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - ru, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        }
        {
            uint256 xx = mulmod(rx, rv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 yy = mulmod(ry, ru, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 zz = mulmod(ru, rv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 xx2 = mulmod(xx, xx, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 yy2 = mulmod(yy, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 xxyy = mulmod(xx, yy, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            uint256 zz2 = mulmod(zz, zz, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
            rx = xxyy + xxyy;
            ru = yy2 - xx2 + 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed;
            ry = xx2 + yy2;
            rv = addmod(zz2 + zz2, 0xffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffda - ru, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
        }
        // Step 6: compare the points
        return mulmod(rx, vu, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed) == mulmod(vx, ru, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed) && mulmod(ry, vv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed) == mulmod(vy, rv, 0x7fffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffff_ffffffed);
    }
}