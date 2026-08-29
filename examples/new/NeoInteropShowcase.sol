// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title NeoInteropShowcase
 * @notice Demonstrates Neo-focused intrinsic usage from Solidity.
 */
contract NeoInteropShowcase {
    mapping(address => uint256) private credits;

    event Credited(address indexed account, uint256 amount, uint256 at);

    function credit(uint256 amount) external {
        require(Runtime.checkWitness(msg.sender), "witness required");
        credits[msg.sender] += amount;

        bytes memory key = abi.encodePacked("credit:", msg.sender);
        Storage.put(key, abi.encode(credits[msg.sender]));

        uint256 timestamp = Runtime.getTime();
        Runtime.notify("Credited", abi.encode(msg.sender, amount, timestamp));
        emit Credited(msg.sender, amount, timestamp);
    }

    function creditOf(address account) external view returns (uint256) {
        bytes memory key = abi.encodePacked("credit:", account);
        bytes memory value = Storage.get(key);
        if (value.length == 0) {
            return 0;
        }
        return abi.decode(value, (uint256));
    }

    function transferGasFromSelf(address to, uint256 amount) external returns (bool) {
        return NativeCalls.gasTransfer(address(this), to, amount, "");
    }

    function gasBalanceViaSyscall(address account) external view returns (uint256) {
        bytes memory ret = Syscalls.contractCall(
            NativeCalls.GAS_CONTRACT,
            "balanceOf",
            abi.encode(account)
        );
        return abi.decode(ret, (uint256));
    }
}
