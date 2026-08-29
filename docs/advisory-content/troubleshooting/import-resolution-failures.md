---
title: "Troubleshooting: Import Resolution Failures"
description: "Import Resolution Failures from Troubleshooting."
---

# Import Resolution Failures

[Back to Troubleshooting](/advisory-content/troubleshooting)

## E4003: Unresolved Import

```
error[E4003]: unresolved import 'libraries/Storage.sol'
  --> MyContract.sol:3:1
   |
3  | import "libraries/Storage.sol";
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = help: pass the devpack root with -I: neo-solc MyContract.sol -I devpack
```

**Cause:** The compiler cannot find the imported file relative to any include path.

**Solutions:**

1. Pass the devpack root as an include path:

   ```bash
   neo-solc MyContract.sol -I devpack -o build/MyContract
   ```

2. Verify the import path matches the devpack directory structure:

   ```
   devpack/
   ├── contracts/    → import "contracts/NativeCalls.sol"
   ├── libraries/    → import "libraries/Storage.sol"
   └── standards/    → import "standards/NEP17.sol"
   ```

3. For multiple include paths, pass `-I` multiple times:
   ```bash
   neo-solc MyContract.sol -I devpack -I ./my-libs -o build/MyContract
   ```

::: tip
Import paths are resolved relative to each `-I` directory in order. The compiler does not use Solidity remappings or package managers.
:::

---
