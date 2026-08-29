---
title: "Native Contracts: StdLib"
description: "StdLib from Native Contracts."
---

# StdLib

[Back to Native Contracts](/internals/native-contracts)

Provides serialization, encoding, and string manipulation utilities. StdLib methods are exposed through `Syscalls.sol`.

## Methods

| Method            | Signature                               | Return     | Safe | Description                                         |
| ----------------- | --------------------------------------- | ---------- | :--: | --------------------------------------------------- |
| `serialize`       | `serialize(bytes)`                      | `bytes`    |  ✅  | Serialize a NeoVM stack item to bytes.              |
| `deserialize`     | `deserialize(bytes)`                    | `bytes`    |  ✅  | Deserialize bytes back to a stack item.             |
| `jsonSerialize`   | `jsonSerialize(bytes)`                  | `bytes`    |  ✅  | Serialize a stack item to JSON.                     |
| `jsonDeserialize` | `jsonDeserialize(bytes)`                | `bytes`    |  ✅  | Deserialize JSON to a stack item.                   |
| `base64Encode`    | `base64Encode(bytes)`                   | `string`   |  ✅  | Base64 encode.                                      |
| `base64Decode`    | `base64Decode(string)`                  | `bytes`    |  ✅  | Base64 decode.                                      |
| `base58Encode`    | `base58Encode(bytes)`                   | `string`   |  ✅  | Base58 encode.                                      |
| `base58Decode`    | `base58Decode(string)`                  | `bytes`    |  ✅  | Base58 decode.                                      |
| `itoa`            | `itoa(int256,uint8)`                    | `string`   |  ✅  | Integer to string (base 10 or 16).                  |
| `atoi`            | `atoi(string,uint8)`                    | `int256`   |  ✅  | String to integer (base 10 or 16).                  |
| `memoryCompare`   | `memoryCompare(bytes,bytes)`            | `int256`   |  ✅  | Lexicographic byte comparison. Returns -1, 0, or 1. |
| `memorySearch`    | `memorySearch(bytes,bytes,int256,bool)` | `int256`   |  ✅  | Search for a byte pattern. Returns index or -1.     |
| `stringSplit`     | `stringSplit(string,string,bool)`       | `string[]` |  ✅  | Split string by separator.                          |
| `strLen`          | `strLen(string)`                        | `uint256`  |  ✅  | String length in text elements (not bytes).         |

## Code Example

```solidity
import "devpack/contracts/Syscalls.sol";

contract DataEncoder {
    function encodeForStorage(uint256 value) public view returns (bytes memory) {
        return Syscalls.serialize(abi.encode(value));
    }

    function toBase64(bytes memory data) public view returns (string memory) {
        return Syscalls.base64Encode(data);
    }

    function numberToHexString(int256 value) public view returns (string memory) {
        return Syscalls.itoa(value, 16);
    }

    function splitCsv(string memory csv) public view returns (string[] memory) {
        return Syscalls.stringSplit(csv, ",", true);
    }
}
```

---
