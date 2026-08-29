---
title: "Native Contracts: Method Token Optimization (CALLT)"
description: "Method Token Optimization (CALLT) from Native Contracts."
---

# Method Token Optimization (CALLT)

[Back to Native Contracts](/internals/native-contracts)

By default, native contract calls are emitted as `System.Contract.Call` syscalls with the target contract hash and method name pushed onto the stack. The `--callt` compiler flag enables an optimization that replaces these with `CALLT` instructions referencing a method token table embedded in the NEF binary.

```bash
neo-solc contract.sol --callt -O3 -o build/contract
```

## How It Works

Without `--callt`:

```
PUSHDATA1 <20-byte contract hash>
PUSHDATA1 <method name string>
PUSHINT8 <call flags>
SYSCALL System.Contract.Call
```

With `--callt`:

```
CALLT <2-byte token index>
```

The NEF header contains a method token table that maps each token index to a `(contractHash, method, parametersCount, hasReturnValue, callFlags)` tuple. The Neo VM resolves the token at load time.

## Benefits

- Smaller bytecode — eliminates inline contract hash and method name strings per call site
- Lower GAS cost — fewer opcodes executed per native call
- Explicit manifest permissions — each method token generates a precise permission entry

::: info
CALLT is a pure optimization. It does not change contract behavior. Contracts compiled with and without `--callt` are functionally identical. Use `--callt` in production builds for best efficiency.
:::

---
