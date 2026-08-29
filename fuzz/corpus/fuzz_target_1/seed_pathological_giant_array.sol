// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract GiantArray {
    // Large static sizes — stresses compile-time size tracking.
    uint256[100000] public storageArr;
    uint8[1000000] public hugeBytes;

    // Nested huge arrays
    uint256[1000][1000] public matrix;

    // Near-max (2^256 - 1 would OOM; use a gentler but still huge)
    uint256[0xFFFFFF] public nearMax;

    function memoryAlloc() external pure returns (uint256) {
        uint256[100000] memory a;
        a[0] = 1;
        a[99999] = 2;
        return a[0] + a[99999];
    }

    function smallerMem() external pure returns (uint256) {
        uint256[10000] memory b;
        return b.length;
    }
}
