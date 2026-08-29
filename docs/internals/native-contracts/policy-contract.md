---
title: "Native Contracts: Policy Contract"
description: "Policy Contract from Native Contracts."
---

# Policy Contract

[Back to Native Contracts](/internals/native-contracts)

Controls network-wide policy parameters. Most setter methods require committee multi-signature authorization.

## Methods

| Method                           | Signature                                              | Return    | Safe | Embedded runtime behavior                                           |
| -------------------------------- | ------------------------------------------------------ | --------- | :--: | ------------------------------------------------------------------- |
| `getFeePerByte`                  | `getFeePerByte()`                                      | `uint256` |  ✅  | Returns current embedded fee-per-byte value.                        |
| `setFeePerByte`                  | `setFeePerByte(uint256)`                               | `void`    |  ❌  | Updates embedded policy state; committee authorization is not modeled. |
| `getExecFeeFactor`               | `getExecFeeFactor()`                                   | `uint32`  |  ✅  | Returns current embedded execution fee factor.                      |
| `setExecFeeFactor`               | `setExecFeeFactor(uint32)`                             | `void`    |  ❌  | Updates embedded execution fee factor.                              |
| `getExecPicoFeeFactor`           | `getExecPicoFeeFactor()`                               | `uint256` |  ✅  | Returns execution fee factor scaled to picoGAS.                     |
| `getStoragePrice`                | `getStoragePrice()`                                    | `uint256` |  ✅  | Returns current embedded storage price.                             |
| `setStoragePrice`                | `setStoragePrice(uint256)`                             | `void`    |  ❌  | Updates embedded storage price.                                     |
| `getMillisecondsPerBlock`        | `getMillisecondsPerBlock()`                            | `uint256` |  ✅  | Returns current embedded milliseconds-per-block value.              |
| `setMillisecondsPerBlock`        | `setMillisecondsPerBlock(uint256)`                     | `void`    |  ❌  | Updates embedded milliseconds-per-block value.                      |
| `getMaxValidUntilBlockIncrement` | `getMaxValidUntilBlockIncrement()`                     | `uint256` |  ✅  | Returns current embedded max valid-until-block increment.           |
| `setMaxValidUntilBlockIncrement` | `setMaxValidUntilBlockIncrement(uint256)`              | `void`    |  ❌  | Updates embedded max valid-until-block increment.                   |
| `getMaxTraceableBlocks`          | `getMaxTraceableBlocks()`                              | `uint256` |  ✅  | Returns current embedded max traceable blocks value.                |
| `setMaxTraceableBlocks`          | `setMaxTraceableBlocks(uint256)`                       | `void`    |  ❌  | Updates embedded max traceable blocks value.                        |
| `getAttributeFee`                | `getAttributeFee(uint8)`                               | `uint256` |  ✅  | Reads fee for the requested attribute type, defaulting to zero.     |
| `setAttributeFee`                | `setAttributeFee(uint8,uint256)`                       | `void`    |  ❌  | Updates embedded attribute-fee map.                                 |
| `isBlocked`                      | `isBlocked(address)`                                   | `bool`    |  ✅  | Checks embedded blocked-account set.                                |
| `blockAccount`                   | `blockAccount(address)`                                | `bool`    |  ❌  | Adds non-empty account to embedded blocked-account set.             |
| `unblockAccount`                 | `unblockAccount(address)`                              | `bool`    |  ❌  | Removes account from embedded blocked-account set.                  |
| `getBlockedAccounts`             | `getBlockedAccounts()`                                 | iterator/array | ✅ | Devpack exposes an iterator wrapper; embedded runtime returns an array of blocked accounts. |
| `setWhitelistFeeContract`        | `setWhitelistFeeContract(address,string,uint8,uint256)` | `void`   |  ❌  | Adds an embedded whitelist-fee contract entry.                      |
| `removeWhitelistFeeContract`     | `removeWhitelistFeeContract(address,string,uint8)`      | `void`   |  ❌  | Removes embedded entries matching the contract hash.                |
| `getWhitelistFeeContracts`       | `getWhitelistFeeContracts()`                           | iterator/array | ✅ | Devpack exposes an iterator wrapper; embedded runtime returns an array of entries. |
| `recoverFund`                    | `recoverFund(address,address)`                         | `bool`    |  ❌  | Embedded runtime is a no-op that returns `true`.                    |

## Code Example

```solidity
import "devpack/contracts/NativeCalls.sol";

contract FeeEstimator {
    function estimateStorageCost(uint256 bytesCount) public view returns (uint256) {
        uint256 pricePerByte = NativeCalls.getStoragePrice();
        return pricePerByte * bytesCount;
    }

    function getNetworkFeeParams() public view returns (uint256 feePerByte, uint32 execFactor) {
        feePerByte = NativeCalls.getFeePerByte();
        execFactor = NativeCalls.getExecFeeFactor();
    }

    function requireNotBlocked(address account) internal view {
        require(!NativeCalls.isBlocked(account), "account is blocked");
    }
}
```

---
