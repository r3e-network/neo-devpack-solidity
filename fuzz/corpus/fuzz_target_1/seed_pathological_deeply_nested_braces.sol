// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract DeepBraces {
    function nested() external pure returns (uint256 r) {
        {{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{{ r = 1; }}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}
    }
    function nestedArray() external pure returns (uint256) {
        uint256[1][1][1][1][1][1][1][1][1][1] memory a;
        return 0;
    }
}
