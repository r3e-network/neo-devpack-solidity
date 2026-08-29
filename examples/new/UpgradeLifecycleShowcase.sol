// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title UpgradeLifecycleShowcase
 * @notice Demonstrates owner+witness gated upgrade and destroy flows on Neo N3.
 *
 * Neo N3 upgrades are performed via ContractManagement native contract calls.
 * Constructor `msg.sender` on Neo deploy is the ContractManagement native contract, so
 * ownership is bootstrapped explicitly after deployment through `bootstrapOwner()`.
 * This sample keeps those operations explicit and tightly authorized.
 */
contract UpgradeLifecycleShowcase {
    address public owner;
    bool private ownerInitialized;
    uint256 public version;

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event ContractUpgraded(uint256 indexed previousVersion, uint256 indexed newVersion);
    event ContractDestroyed(address indexed by);

    error AlreadyInitialized();
    error InvalidWitness();
    error InvalidOwner();
    error InvalidVersion();
    error EmptyArtifact();

    modifier onlyOwnerWithWitness() {
        if (!Runtime.checkWitness(owner)) revert InvalidWitness();
        _;
    }

    constructor(uint256 initialVersion) {
        version = initialVersion == 0 ? 1 : initialVersion;
    }

    function bootstrapOwner(address initialOwner) external {
        if (ownerInitialized) revert AlreadyInitialized();
        if (!Runtime.checkWitness(initialOwner)) revert InvalidWitness();

        owner = initialOwner;
        ownerInitialized = true;
    }

    function transferOwnership(address newOwner) external onlyOwnerWithWitness {
        owner = newOwner;
    }

    function upgrade(
        bytes calldata nef,
        bytes calldata manifest,
        uint256 newVersion
    ) external onlyOwnerWithWitness {
        if (nef.length == 0 || manifest.length == 0) revert EmptyArtifact();
        if (newVersion <= version) revert InvalidVersion();

        uint256 previousVersion = version;
        version = newVersion;

        NativeCalls.updateContract(nef, manifest);
    }

    function destroyContract() external onlyOwnerWithWitness {
        NativeCalls.destroyContract();
    }

    function gasBalance() external view returns (uint256) {
        return NativeCalls.gasBalanceOf(address(this));
    }
}
