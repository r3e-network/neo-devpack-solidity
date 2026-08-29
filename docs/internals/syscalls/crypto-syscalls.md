---
title: "Syscalls: Crypto Syscalls"
description: "Crypto Syscalls from Syscalls."
---

# Crypto Syscalls

[Back to Syscalls](/internals/syscalls)

## Signature Syscalls

Crypto syscalls verify signatures against the current transaction's script container.

| Syscall Name                  | Gas Cost | Description                | Devpack Wrapper                         |
| ----------------------------- | -------: | -------------------------- | --------------------------------------- |
| `System.Crypto.CheckSig`      |    1,000 | Verify single signature    | `Syscalls.checkSig(pubkey, sig)`        |
| `System.Crypto.CheckMultisig` |    1,000 | Verify multiple signatures | `Syscalls.checkMultisig(pubkeys, sigs)` |

These syscalls verify signatures against the hash of the current script container (transaction). For general-purpose signature verification with arbitrary messages, use the `CryptoLib` native contract via `Syscalls.verifyWithECDsa()`.

## Example Usage

```solidity
import "devpack/contracts/Syscalls.sol";

contract Verifier {
    // Verify a single signature against the current transaction
    function verifySigner(bytes memory publicKey, bytes memory signature)
        external view returns (bool)
    {
        return Syscalls.checkSig(publicKey, signature);
    }

    // For arbitrary message verification, use CryptoLib (native contract)
    function verifyMessage(
        bytes32 messageHash,
        bytes memory publicKey,
        bytes memory signature
    ) external view returns (bool) {
        return Syscalls.verifyWithECDsa(
            messageHash, publicKey, signature,
            Syscalls.SECP256K1_SHA256  // or SECP256R1_SHA256
        );
    }
}
```

## Native CryptoLib

::: info
Most cryptographic operations in Neo N3 are handled by the `CryptoLib` native contract (SHA256, RIPEMD160, keccak256, ECDSA, Ed25519, BLS12-381, Murmur32), which is invoked via `System.Contract.Call` rather than dedicated syscalls. The two `System.Crypto.*` syscalls exist specifically for transaction-level signature verification used in consensus and verification triggers.
:::
