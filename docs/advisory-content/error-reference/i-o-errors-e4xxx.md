---
title: "Error Reference: I/O Errors (E4xxx)"
description: "I/O Errors (E4xxx) from Error Reference."
---

# I/O Errors (E4xxx)

[Back to Error Reference](/advisory-content/error-reference)

| Code    | Name             | Description              | Common Cause                                |
| ------- | ---------------- | ------------------------ | ------------------------------------------- |
| `E4001` | FileNotFound     | File not found           | Source file path does not exist             |
| `E4002` | PermissionDenied | Permission denied        | Cannot read source file or write output     |
| `E4003` | ImportNotFound   | Import not found         | Imported file not found in any include path |
| `E4004` | CircularImport   | Circular import detected | File A imports B which imports A            |

## Example: E4003 ImportNotFound

```
error[E4003]: import not found: '@neo/devpack/NeoToken.sol'
  --> MyContract.sol:3:1
   |
 3 | import "@neo/devpack/NeoToken.sol";
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

**Fix:** Add the correct include path:

```bash
neo-solc contract.sol -I devpack -I ./node_modules -o build/
```

**Checklist:**

- Verify the import path matches the actual file location
- Check that `-I` paths are relative to the working directory or absolute
- Ensure no typos in the import statement
- For devpack imports, ensure the `devpack/` directory is present in the project

## Example: E4004 CircularImport

```
error[E4004]: circular import detected: A.sol -> B.sol -> A.sol
```

**Fix:** Break the circular dependency by extracting shared types into a third file that both can import, or restructure the contract hierarchy.
