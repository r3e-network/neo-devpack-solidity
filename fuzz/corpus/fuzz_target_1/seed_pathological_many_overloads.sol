// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract ManyOverloads {
    function doIt(uint256 p0) external pure returns (uint256) { return 0; }
    function doIt(uint256 p0, uint256 p1) external pure returns (uint256) { return 1; }
    function doIt(uint256 p0, uint256 p1, uint256 p2) external pure returns (uint256) { return 2; }
    function doIt(uint256 p0, uint256 p1, uint256 p2, uint256 p3) external pure returns (uint256) { return 3; }
    function doIt(uint256 p0, uint256 p1, uint256 p2, uint256 p3, uint256 p4) external pure returns (uint256) { return 4; }
    function doIt(uint256 p0, uint256 p1, uint256 p2, uint256 p3, uint256 p4, uint256 p5) external pure returns (uint256) { return 5; }
    function doIt(uint256 p0, uint256 p1, uint256 p2, uint256 p3, uint256 p4, uint256 p5, uint256 p6) external pure returns (uint256) { return 6; }
    function doIt(uint256 p0, uint256 p1, uint256 p2, uint256 p3, uint256 p4, uint256 p5, uint256 p6, uint256 p7) external pure returns (uint256) { return 7; }
    function doIt(uint256 p0, uint256 p1, uint256 p2, uint256 p3, uint256 p4, uint256 p5, uint256 p6, uint256 p7, uint256 p8) external pure returns (uint256) { return 8; }
    function doIt(uint256 p0, uint256 p1, uint256 p2, uint256 p3, uint256 p4, uint256 p5, uint256 p6, uint256 p7, uint256 p8, uint256 p9) external pure returns (uint256) { return 9; }
    function doIt(address a0) external pure returns (uint256) { return 10; }
    function doIt(address a0, address a1) external pure returns (uint256) { return 11; }
    function doIt(address a0, address a1, address a2) external pure returns (uint256) { return 12; }
    function doIt(address a0, address a1, address a2, address a3) external pure returns (uint256) { return 13; }
    function doIt(address a0, address a1, address a2, address a3, address a4) external pure returns (uint256) { return 14; }
    function doIt(address a0, address a1, address a2, address a3, address a4, address a5) external pure returns (uint256) { return 15; }
    function doIt(address a0, address a1, address a2, address a3, address a4, address a5, address a6) external pure returns (uint256) { return 16; }
    function doIt(address a0, address a1, address a2, address a3, address a4, address a5, address a6, address a7) external pure returns (uint256) { return 17; }
    function doIt(address a0, address a1, address a2, address a3, address a4, address a5, address a6, address a7, address a8) external pure returns (uint256) { return 18; }
    function doIt(address a0, address a1, address a2, address a3, address a4, address a5, address a6, address a7, address a8, address a9) external pure returns (uint256) { return 19; }
    function doIt(bytes32 b0) external pure returns (uint256) { return 20; }
    function doIt(bytes32 b0, bytes32 b1) external pure returns (uint256) { return 21; }
    function doIt(bytes32 b0, bytes32 b1, bytes32 b2) external pure returns (uint256) { return 22; }
    function doIt(bytes32 b0, bytes32 b1, bytes32 b2, bytes32 b3) external pure returns (uint256) { return 23; }
    function doIt(bytes32 b0, bytes32 b1, bytes32 b2, bytes32 b3, bytes32 b4) external pure returns (uint256) { return 24; }
    function doIt(bytes32 b0, bytes32 b1, bytes32 b2, bytes32 b3, bytes32 b4, bytes32 b5) external pure returns (uint256) { return 25; }
    function doIt(bytes32 b0, bytes32 b1, bytes32 b2, bytes32 b3, bytes32 b4, bytes32 b5, bytes32 b6) external pure returns (uint256) { return 26; }
    function doIt(bytes32 b0, bytes32 b1, bytes32 b2, bytes32 b3, bytes32 b4, bytes32 b5, bytes32 b6, bytes32 b7) external pure returns (uint256) { return 27; }
    function doIt(bytes32 b0, bytes32 b1, bytes32 b2, bytes32 b3, bytes32 b4, bytes32 b5, bytes32 b6, bytes32 b7, bytes32 b8) external pure returns (uint256) { return 28; }
    function doIt(bytes32 b0, bytes32 b1, bytes32 b2, bytes32 b3, bytes32 b4, bytes32 b5, bytes32 b6, bytes32 b7, bytes32 b8, bytes32 b9) external pure returns (uint256) { return 29; }
    function doIt(bool fl0) external pure returns (uint256) { return 30; }
    function doIt(bool fl0, bool fl1) external pure returns (uint256) { return 31; }
    function doIt(bool fl0, bool fl1, bool fl2) external pure returns (uint256) { return 32; }
    function doIt(bool fl0, bool fl1, bool fl2, bool fl3) external pure returns (uint256) { return 33; }
    function doIt(bool fl0, bool fl1, bool fl2, bool fl3, bool fl4) external pure returns (uint256) { return 34; }
    function doIt(bool fl0, bool fl1, bool fl2, bool fl3, bool fl4, bool fl5) external pure returns (uint256) { return 35; }
    function doIt(bool fl0, bool fl1, bool fl2, bool fl3, bool fl4, bool fl5, bool fl6) external pure returns (uint256) { return 36; }
    function doIt(bool fl0, bool fl1, bool fl2, bool fl3, bool fl4, bool fl5, bool fl6, bool fl7) external pure returns (uint256) { return 37; }
    function doIt(bool fl0, bool fl1, bool fl2, bool fl3, bool fl4, bool fl5, bool fl6, bool fl7, bool fl8) external pure returns (uint256) { return 38; }
    function doIt(bool fl0, bool fl1, bool fl2, bool fl3, bool fl4, bool fl5, bool fl6, bool fl7, bool fl8, bool fl9) external pure returns (uint256) { return 39; }
    function doIt(int256 n0) external pure returns (uint256) { return 40; }
    function doIt(int256 n0, int256 n1) external pure returns (uint256) { return 41; }
    function doIt(int256 n0, int256 n1, int256 n2) external pure returns (uint256) { return 42; }
    function doIt(int256 n0, int256 n1, int256 n2, int256 n3) external pure returns (uint256) { return 43; }
    function doIt(int256 n0, int256 n1, int256 n2, int256 n3, int256 n4) external pure returns (uint256) { return 44; }
    function doIt(int256 n0, int256 n1, int256 n2, int256 n3, int256 n4, int256 n5) external pure returns (uint256) { return 45; }
    function doIt(int256 n0, int256 n1, int256 n2, int256 n3, int256 n4, int256 n5, int256 n6) external pure returns (uint256) { return 46; }
    function doIt(int256 n0, int256 n1, int256 n2, int256 n3, int256 n4, int256 n5, int256 n6, int256 n7) external pure returns (uint256) { return 47; }
    function doIt(int256 n0, int256 n1, int256 n2, int256 n3, int256 n4, int256 n5, int256 n6, int256 n7, int256 n8) external pure returns (uint256) { return 48; }
    function doIt(int256 n0, int256 n1, int256 n2, int256 n3, int256 n4, int256 n5, int256 n6, int256 n7, int256 n8, int256 n9) external pure returns (uint256) { return 49; }
}
