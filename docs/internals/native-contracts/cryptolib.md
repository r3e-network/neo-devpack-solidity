---
title: "Native Contracts: CryptoLib"
description: "CryptoLib from Native Contracts."
---

# CryptoLib

[Back to Native Contracts](/internals/native-contracts)

Provides cryptographic hash functions, signature verification, and BLS12-381 curve operations. CryptoLib methods are exposed through `Syscalls.sol` rather than `NativeCalls.sol`.

## Methods

| Method                | Signature                                    | Return    | Safe | Description                                     |
| --------------------- | -------------------------------------------- | --------- | :--: | ----------------------------------------------- |
| `sha256`              | `sha256(bytes)`                              | `bytes32` |  ✅  | SHA-256 hash.                                   |
| `ripemd160`           | `ripemd160(bytes)`                           | `bytes20` |  ✅  | RIPEMD-160 hash.                                |
| `keccak256`           | `keccak256(bytes)`                           | `bytes32` |  ✅  | Keccak-256 hash (added at Cockatrice hardfork). |
| `murmur32`            | `murmur32(bytes,uint32)`                     | `bytes4`  |  ✅  | Murmur3 32-bit hash with seed.                  |
| `verifyWithECDsa`     | `verifyWithECDsa(bytes32,bytes,bytes,uint8)` | `bool`    |  ✅  | ECDSA signature verification.                   |
| `bls12381Serialize`   | `bls12381Serialize(bytes)`                   | `bytes`   |  ✅  | Serialize a BLS12-381 point.                    |
| `bls12381Deserialize` | `bls12381Deserialize(bytes)`                 | `bytes`   |  ✅  | Deserialize a BLS12-381 point.                  |
| `bls12381Equal`       | `bls12381Equal(bytes,bytes)`                 | `bool`    |  ✅  | Compare two BLS12-381 points.                   |
| `bls12381Add`         | `bls12381Add(bytes,bytes)`                   | `bytes`   |  ✅  | Add two BLS12-381 points.                       |
| `bls12381Mul`         | `bls12381Mul(bytes,bytes,bool)`              | `bytes`   |  ✅  | Multiply BLS12-381 point by scalar.             |
| `bls12381Pairing`     | `bls12381Pairing(bytes,bytes)`               | `bytes`   |  ✅  | BLS12-381 pairing check.                        |

## Curve Support

The `verifyWithECDsa` method accepts a `curve` parameter that selects both the elliptic curve and the hash algorithm:

| Value | Constant              | Curve     | Hash       | Use Case                       |
| :---: | --------------------- | --------- | ---------- | ------------------------------ |
| `22`  | `SECP256K1_SHA256`    | secp256k1 | SHA-256    | Bitcoin-compatible signatures  |
| `23`  | `SECP256R1_SHA256`    | secp256r1 | SHA-256    | Neo-native signatures          |
| `122` | `SECP256K1_KECCAK256` | secp256k1 | Keccak-256 | Ethereum-compatible signatures |
| `123` | `SECP256R1_KECCAK256` | secp256r1 | Keccak-256 | Neo + Keccak signatures        |

## Code Example

```solidity
import "devpack/contracts/Syscalls.sol";

contract SignatureVerifier {
    function verifyEthSignature(
        bytes32 messageHash,
        bytes memory publicKey,
        bytes memory signature
    ) public view returns (bool) {
        // secp256k1 + Keccak-256 for Ethereum compatibility
        return Syscalls.verifyWithECDsa(
            messageHash, publicKey, signature,
            Syscalls.SECP256K1_KECCAK256
        );
    }

    function verifyNeoSignature(
        bytes32 messageHash,
        bytes memory publicKey,
        bytes memory signature
    ) public view returns (bool) {
        // secp256r1 + SHA-256 for Neo-native signatures
        return Syscalls.verifyWithECDsa(
            messageHash, publicKey, signature,
            Syscalls.SECP256R1_SHA256
        );
    }

    function hashData(bytes memory data) public view returns (bytes32) {
        return Syscalls.sha256(data);
    }
}
```

::: info
Solidity's built-in `keccak256()` and `sha256()` are automatically lowered to `CryptoLib.keccak256` and `CryptoLib.sha256` native contract calls by the compiler. The `Syscalls.*` wrappers are provided for explicit usage when you want to call through the devpack namespace directly.
:::

---
