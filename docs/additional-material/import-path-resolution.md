# Import Path Resolution

In order to be able to support reproducible builds on all platforms, the Solidity compiler has to abstract away the details of the filesystem where source files are stored. Paths used in imports must work the same way everywhere while the command-line interface must be able to work with platform-specific paths to provide good user experience. 

Neo DevPack for Solidity uses the same import path resolution strategy as mainline Solidity.

## Virtual Filesystem

The compiler maintains an internal database (virtual filesystem or VFS for short) where each source unit is assigned a unique source unit name. When you use an import statement:

```solidity
import "./MyContract.sol";
import {NativeCalls} from "@neo/NativeCalls.sol";
```
The compiler attempts to resolve these string literals into source unit names inside the VFS.

## Base Path and Include Paths

The compiler handles physical file paths using two main parameters:
1. `--base-path`: The root directory for the project. By default, this is the current working directory.
2. `--include-path` (`-I`): Additional directories to search for imports.

When an import is not found relative to the current file, the compiler scans the include paths.

### Devpack Resolution

In Neo DevPack for Solidity, you typically import standard Neo interfaces from the devpack:

```solidity
import {Runtime} from "@neo/Runtime.sol";
```

For this to resolve successfully, you must compile with the `-I` flag pointing to your devpack directory:
```bash
neo-solc MyContract.sol -I devpack
```
This instructs the compiler to look inside `devpack/@neo/Runtime.sol` when attempting to resolve the import.

## URL Imports

::: tip 💡 NeoVM Difference
Some Ethereum frameworks (like Remix) allow importing directly from GitHub URLs (e.g. `import "https://github.com/..."`). **Neo DevPack for Solidity's command-line compiler does not support absolute URL imports.** You must download dependencies locally and provide their paths via `-I`.
:::