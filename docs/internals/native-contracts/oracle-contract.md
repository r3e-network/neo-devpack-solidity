---
title: "Native Contracts: Oracle Contract"
description: "Oracle Contract from Native Contracts."
---

# Oracle Contract

[Back to Native Contracts](/internals/native-contracts)

Provides off-chain data access through a request/callback pattern. Oracle nodes fetch external data and deliver results back to the requesting contract.

## Methods

| Method     | Signature                                     | Return    | Safe | Description                                      |
| ---------- | --------------------------------------------- | --------- | :--: | ------------------------------------------------ |
| `request`  | `request(string,string,string,bytes,uint256)` | `void`    |  ❌  | Submit an oracle data request.                   |
| `getPrice` | `getPrice()`                                  | `uint256` |  ✅  | Base GAS cost per oracle request.                |
| `setPrice` | `setPrice(uint256)`                           | `void`    |  ❌  | Set oracle price (committee only).               |
| `finish`   | `finish()`                                    | `void`    |  ❌  | Complete an oracle response (oracle nodes only). |
| `verify`   | `verify()`                                    | `bool`    |  ✅  | Verify an oracle response transaction.           |

## Embedded Runtime Behavior

The embedded runtime is deterministic and offline. It records `request`
arguments, assigns incrementing request IDs for direct native calls, and keeps
local oracle price state for tests. It does not fetch external URLs, run JSONPath
filters, contact oracle nodes, or deliver live callbacks.

`getPrice()` starts at `50_000_000` GAS fractions, `setPrice(uint256)` updates
that local value without committee authorization checks, `finish()` is a no-op,
and `verify()` returns `true`.

Use Neo-Express or TestNet when the behavior under test depends on real oracle
network delivery.

## Callback Pattern

The Oracle native contract uses an asynchronous request/callback model:

1. Your contract calls `Oracle.request(url, filter, callbackMethod, userData, gasForResponse)`.
2. Oracle nodes fetch the URL, apply the JSONPath `filter`, and invoke `callbackMethod` on your contract.
3. The callback receives `(string url, bytes userData, int code, bytes result)`.

```solidity
import "devpack/contracts/NativeCalls.sol";

contract PriceOracle {
    uint256 public lastPrice;

    function requestPrice() public {
        NativeCalls.requestOracleData(
            "https://api.example.com/price",  // URL
            "$.neo.usd",                       // JSONPath filter
            "onPriceResponse",                 // callback method name
            "",                                // user data
            100_000_000                        // 1 GAS for response
        );
    }

    /// @dev Called by the Oracle native contract
    function onPriceResponse(
        string calldata url,
        bytes calldata userData,
        uint256 code,
        bytes calldata result
    ) external {
        require(msg.sender == NativeCalls.ORACLE_CONTRACT, "unauthorized");
        if (code == 0) { // Success
            lastPrice = abi.decode(result, (uint256));
        }
    }
}
```

::: tip
The devpack includes `OracleService.sol` — a convenience wrapper that manages request IDs, stores responses, and forwards callbacks through a fixed `onOracleResponse` method name. This avoids wildcard manifest permissions that a dynamic callback name would require.
:::

::: warning
Oracle requests are not free. Each request costs at least `Oracle.getPrice()` GAS plus the `gasForResponse` budget you specify. The callback method name becomes a manifest permission entry — use a fixed name to avoid wildcard permissions.
:::

---
