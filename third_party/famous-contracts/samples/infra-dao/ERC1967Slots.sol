// SPDX-License-Identifier: MIT
// EIP-1967: Standard Proxy Storage Slots
// Reference: https://eips.ethereum.org/EIPS/eip-1967
// Self-contained demo: the canonical EIP-1967 storage slots and an upgrade-admin
// registry that stores implementation/admin/beacon addresses at those exact slots.
// Note: EVM `delegatecall` fallback proxying has no NeoVM equivalent, so this sample
// demonstrates the slot bookkeeping and admin-gated upgrade logic (the part that is
// portable), not raw delegatecall dispatch.
pragma solidity ^0.8.0;

contract ERC1967Slots {
    // bytes32(uint256(keccak256('eip1967.proxy.implementation')) - 1)
    bytes32 internal constant IMPLEMENTATION_SLOT =
        0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc;

    // bytes32(uint256(keccak256('eip1967.proxy.admin')) - 1)
    bytes32 internal constant ADMIN_SLOT =
        0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103;

    // bytes32(uint256(keccak256('eip1967.proxy.beacon')) - 1)
    bytes32 internal constant BEACON_SLOT =
        0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50;

    // Slot-indexed storage standing in for EVM sstore/sload at fixed slots.
    mapping(bytes32 => address) private _slot;

    event Upgraded(address indexed implementation);
    event AdminChanged(address previousAdmin, address newAdmin);
    event BeaconUpgraded(address indexed beacon);

    constructor() {
        _slot[ADMIN_SLOT] = msg.sender;
    }

    modifier onlyAdmin() {
        require(msg.sender == _slot[ADMIN_SLOT], "ERC1967: caller is not admin");
        _;
    }

    function implementation() external view returns (address) {
        return _slot[IMPLEMENTATION_SLOT];
    }

    function admin() external view returns (address) {
        return _slot[ADMIN_SLOT];
    }

    function beacon() external view returns (address) {
        return _slot[BEACON_SLOT];
    }

    function upgradeTo(address newImplementation) external onlyAdmin {
        require(newImplementation != address(0), "ERC1967: zero implementation");
        _slot[IMPLEMENTATION_SLOT] = newImplementation;
        emit Upgraded(newImplementation);
    }

    function changeAdmin(address newAdmin) external onlyAdmin {
        require(newAdmin != address(0), "ERC1967: zero admin");
        address previous = _slot[ADMIN_SLOT];
        _slot[ADMIN_SLOT] = newAdmin;
        emit AdminChanged(previous, newAdmin);
    }

    function upgradeBeaconTo(address newBeacon) external onlyAdmin {
        require(newBeacon != address(0), "ERC1967: zero beacon");
        _slot[BEACON_SLOT] = newBeacon;
        emit BeaconUpgraded(newBeacon);
    }
}
