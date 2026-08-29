// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title MultiSig — ERC-1271 contract-signature reference, compiled to Neo N3.
contract MultiSig {
    string public buildTag = "multisig-v1";
    bytes4 private constant MAGIC = 0x1626ba7e;

    address[] public owners;
    uint256 public threshold;
    address public deployer;

    function claimDeployer() public {
        require(deployer == address(0), "MultiSig: already claimed");
        deployer = msg.sender;
    }

    function setup(address[] memory owners_, uint256 threshold_) public {
        require(deployer != address(0), "MultiSig: unclaimed");
        require(msg.sender == deployer, "MultiSig: admin only");
        require(owners_.length > 0 && threshold_ > 0 && threshold_ <= owners_.length, "MultiSig: bad params");
        owners = owners_;
        threshold = threshold_;
    }

    function isOwner(address a) public view returns (bool) {
        for (uint i; i < owners.length; ++i) if (owners[i] == a) return true;
        return false;
    }

    function ownerCount() public view returns (uint256) { return owners.length; }

    /// ERC-1271 isValidSignature — returns MAGIC iff the ECDSA signer is one of
    /// the configured owners.
    function isValidSignature(bytes32 hash, bytes memory sig)
        public view returns (bytes4)
    {
        if (sig.length != 65) return 0xffffffff;

        bytes32 r;
        bytes32 s;
        uint8 v;
        assembly {
            r := mload(add(sig, 32))
            s := mload(add(sig, 64))
            v := byte(0, mload(add(sig, 96)))
        }
        if (v < 27) v += 27;
        address signer = ecrecover(hash, v, r, s);
        if (signer != address(0) && isOwner(signer)) return MAGIC;
        return 0xffffffff;
    }
}
