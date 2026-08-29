# Neo DevPack for Solidity Error Reference

This document provides a comprehensive reference for error and warning codes produced by the Neo DevPack for Solidity compiler.

## Error Codes

### Compilation Errors

| Code                        | Description                | Example                                     |
| --------------------------- | -------------------------- | ------------------------------------------- |
| `VALIDATION_ERROR`          | Contract validation failed | Missing constructor, invalid state variable |
| `IR_GENERATION_ERROR`       | IR generation failed       | Unsupported Solidity feature                |
| `MANIFEST_GENERATION_ERROR` | Manifest creation failed   | Invalid permissions configuration           |
| `GENERIC_ERROR`             | General compilation error  | Unknown error                               |
| `IO_ERROR`                  | File system error          | Cannot read input file                      |

### Warning Codes

| Code                         | Description                                   | Example                           |
| ---------------------------- | --------------------------------------------- | --------------------------------- |
| `COMPILER_WARNING`           | General compiler warning                      | Unused function parameter         |
| `NEF_SOURCE_TRUNCATED`       | NEF source field truncated                    | Source exceeds 240 bytes          |
| `MANIFEST_FULL_WILDCARD`     | Full wildcard permission                      | `{"contract":"*","methods":"*"}`  |
| `MANIFEST_WILDCARD_CONTRACT` | Wildcard contract permission                  | `{"contract":"*",...}`            |
| `MANIFEST_WILDCARD_METHODS`  | Wildcard method permission                    | `{...,"methods":"*"}`             |
| `W_USER_DEFINED_OPERATOR`    | User-defined operators not dispatched         | `using {add as +} for Int global` |
| `W_TRANSIENT_PERSISTED`      | `transient` state variable persisted on NeoVM | `uint256 transient t;`            |

## Common Errors and Solutions

### 1. Import Resolution Errors

**Error:**

```
error: failed to resolve imports: no such file or directory
```

**Solution:**

```bash
# Use -I flag to specify include paths
neo-solc contract.sol -I devpack -o build/

# Or use relative paths
neo-solc ./contracts/MyContract.sol -I ./lib -o ./build/
```

### 2. Permission Errors

**Warning:**

```
warning (COMPILER_WARNING): contract 'MyContract' requires wildcard contract manifest permissions
```

**Solution:**

```bash
# Use explicit permissions
neo-solc contract.sol --manifest-permissions permissions.json -o build/

# Or use strict mode
neo-solc contract.sol --deny-wildcard-contracts --deny-wildcard-methods -o build/
```

### 3. Type Errors

**Error:**

```
error (SemanticError): type mismatch: expected uint256, found address
```

**Solution:**
Ensure you're using the correct types. In Solidity, addresses and integers are not interchangeable.

### 4. Storage Layout Errors

**Error:**

```
error (SemanticError): dynamic types not supported in storage
```

**Solution:**
Use fixed-size arrays or mappings instead of dynamic arrays for storage variables.

### 5. Constructor Errors

**Error:**

```
error: deployment data does not match constructor parameters
```

**Solution:**
Pass constructor arguments as JSON array in deployment:

```bash
neo-cli contract deploy contract.nef contract.manifest.json --data '[1000000]'
```

## Exit Codes

| Code | Description       |
| ---- | ----------------- |
| 0    | Success           |
| 1    | Compilation error |
| 2    | Invalid arguments |
| 3    | File not found    |
| 4    | Permission denied |

## JSON Output Format

Use `--json-errors` for machine-readable diagnostics:

```json
{
  "component": "neo-devpack-solidity",
  "severity": "error",
  "type": "CompilerError",
  "code": "VALIDATION_ERROR",
  "message": "Storage variable 'balances' must be explicitly typed",
  "formattedMessage": "Storage variable 'balances' must be explicitly typed",
  "location": {
    "file": "MyContract.sol",
    "line": 15,
    "column": 5
  }
}
```

## Troubleshooting Guide

### General Issues

1. **Compiler not found**
   - Run `cargo build --release` to build the compiler
   - Ensure `target/release/neo-solc` is in your PATH

2. **Slow compilation**
   - Use optimization flags: `neo-solc contract.sol -O2`
   - Use `--format nef` or `--format manifest` when you only need one artifact type

3. **Large bytecode**
   - Use higher optimization levels
   - Remove unused functions
   - Use external libraries instead of inlining

### Platform-Specific Issues

#### Linux

- Ensure libclang is installed for solang-parser: `apt-get install libclang-dev`

#### macOS

- Xcode command line tools required: `xcode-select --install`

#### Windows

- Visual Studio Build Tools required
- Ensure Rust toolchain is installed with MSVC target

## Getting Help

- Check [Getting Started](/basics/introduction-to-smart-contracts) for quick start
- Review [examples](https://github.com/r3e-network/neo-devpack-solidity/tree/main/examples) for working contracts
- Report issues: https://github.com/r3e-network/neo-devpack-solidity/issues
