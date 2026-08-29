// SPDX-License-Identifier: MIT
// A self-contained CREATE2 deterministic-address helper.
// Reference (EIP-1014, CREATE2): https://eips.ethereum.org/EIPS/eip-1014
// Reference (OZ Create2): https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v4.9.3/contracts/utils/Create2.sol
// EVM's raw `create2` opcode has no NeoVM equivalent, so this sample implements the
// portable half: the deterministic address-derivation formula
//   address = last20bytes( keccak256(0xff ++ deployer ++ salt ++ keccak256(init_code)) )
// plus a registry that tracks salts a deployer has already claimed. This is the piece
// people actually reason about ("counterfactual" addresses) and it is dependency-free.
pragma solidity ^0.8.0;

contract Create2Factory {
    // deployer => salt => already used
    mapping(address => mapping(bytes32 => bool)) public usedSalt;

    event Reserved(address indexed deployer, bytes32 indexed salt, address predicted);

    /// @dev Compute the CREATE2 address for `salt` and a contract whose init code
    ///      hashes to `bytecodeHash`, deployed from `deployer`.
    function computeAddress(
        bytes32 salt,
        bytes32 bytecodeHash,
        address deployer
    ) public pure returns (address) {
        bytes32 digest = keccak256(abi.encodePacked(bytes1(0xff), deployer, salt, bytecodeHash));
        return address(uint160(uint256(digest)));
    }

    /// @dev Convenience overload using this factory as the deployer.
    function computeAddress(bytes32 salt, bytes32 bytecodeHash) external view returns (address) {
        return computeAddress(salt, bytecodeHash, address(this));
    }

    /// @dev Reserve a salt so it cannot be reused for a different deployment.
    function reserve(bytes32 salt, bytes32 bytecodeHash) external returns (address predicted) {
        require(!usedSalt[msg.sender][salt], "Create2: salt already used");
        usedSalt[msg.sender][salt] = true;
        predicted = computeAddress(salt, bytecodeHash, address(this));
        emit Reserved(msg.sender, salt, predicted);
    }

    function isSaltUsed(address deployer, bytes32 salt) external view returns (bool) {
        return usedSalt[deployer][salt];
    }
}
