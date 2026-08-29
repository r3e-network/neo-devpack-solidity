---
title: "Solidity Feature Support: E. OOP Features"
description: "E. OOP Features from Solidity Feature Support."
---

# E. OOP Features

[Back to Solidity Feature Support](/solidity/feature-support)

| Feature              | Status | Notes                                                                                                                  |
| -------------------- | :----: | ---------------------------------------------------------------------------------------------------------------------- |
| Single inheritance   |   ✅   | C3 linearization with `flatten_contract_inheritance`.                                                                  |
| Multiple inheritance |   ✅   | Diamond inheritance detected. Constructor arg conflicts reported.                                                      |
| `interface`          |   ✅   | Interface types tracked. Methods validated.                                                                            |
| `abstract contract`  |   ✅   | Unimplemented functions detected. Non-abstract contracts get actionable errors. A concrete contract may hold an abstract-contract-typed field without being forced abstract itself. |
| `library`            |   ⚠️   | Builtin devpack libraries are compiler intrinsics. User-defined libraries are merged/inlined into consuming contracts. |
| `using X for Y`      |   ✅   | Library member-call syntax fully supported. `using X for *` and `using {f,g} for T` included.                          |
| `super` keyword      |   ✅   | Supported via inheritance flattening with `__super_` method preservation.                                              |
| `is` (inheritance)   |   ✅   | Inheritance specifiers fully processed.                                                                                |
| Constructor chaining |   ✅   | Base constructor arguments resolved from inheritance specifiers.                                                       |
| Event inheritance    |   ✅   | Events inherited from interfaces/bases are collected recursively and resolve across cross-contract `new`. A library's own event declarations are carried into the consuming contract. |

## Partial OOP details

**`library`** — The devpack ships built-in libraries (`Runtime`, `Storage`, `Syscalls`, etc.) that are compiler intrinsics — they lower directly to syscalls and native contract calls. User-defined libraries are merged into the consuming contract. `internal` calls work directly, and `public` / `external` library functions are accepted but normalized to internal helpers with warnings. A library's own event declarations are carried into the consuming contract, so a contract can `emit` an event declared in a library it uses. Libraries still cannot maintain mutable state or behave like separately deployed/linkable EVM libraries.

```solidity
// ✅ Works — using devpack intrinsic library
import "devpack/libraries/Runtime.sol";
require(Runtime.checkWitness(sender), "unauthorized");

// ✅ Works — user-defined library with internal functions
library MathLib {
    function add(uint256 a, uint256 b) internal pure returns (uint256) {
        return a + b;
    }
}

using MathLib for uint256;
uint256 result = x.add(y);
```

---
