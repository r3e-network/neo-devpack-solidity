// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract Empty {}

interface IEmpty {}

library LEmpty {}

abstract contract AEmpty {}

contract OnlyConstructor { constructor() {} }

contract OnlyFallback { fallback() external {} }

contract OnlyReceive { receive() external payable {} }
