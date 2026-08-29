# Analysing the Compiler Output

When you compile a Solidity contract using `neo-solc`, the output differs significantly from standard `solc`.

::: tip 💡 NeoVM Difference
Standard Solidity outputs an EVM binary (often represented as a hex string) and an Ethereum ABI JSON. **Neo DevPack for Solidity outputs a `.nef` (Neo Executable Format) binary file and a `.manifest.json` file.**
:::

## The NEF File (`.nef`)

The NEF file is the compiled bytecode executable deployed to the Neo blockchain. It is a strictly formatted binary file containing:

1. **Magic Header**: Identifies the file as a Neo N3 executable (`0x3346454E`).
2. **Compiler Name**: A 64-byte string identifying the compiler (for example, `neo-devpack-solidity-0.27.0`).
3. **Source/Version Context**: Metadata tracking the compiler version used.
4. **Method Tokens**: A table mapping external contract calls to optimize runtime execution (if `--callt` is used).
5. **Script**: The actual NeoVM opcodes representing your contract logic.
6. **Checksum**: A SHA-256 checksum to ensure file integrity.

You can disassemble the NEF file during compilation by passing the `-f assembly` flag:

```bash
neo-solc MyContract.sol -O2 -f assembly -o build/MyContract
```

## The Manifest File (`.manifest.json`)

The manifest file is a comprehensive JSON document describing how the Neo blockchain and other contracts can interact with your executable.

Key sections include:

- **`abi`**: Describes the methods and events the contract exposes.
  - `methods`: Arrays of functions, their parameters (using Neo types like `Hash160`, `Integer`), and `safe` flags (equivalent to `view`/`pure`).
  - `events`: Notifications the contract emits.
- **`permissions`**: Defines which external contracts and methods this contract is allowed to call. This is critical for security.
- **`supportedstandards`**: Lists the NEP standards the contract implements (e.g., `NEP-17`, `NEP-11`), which are automatically inferred by the compiler.
- **`trusts`**: Specifies which contracts are allowed to invoke this contract without user warnings.
- **`extra`**: Custom metadata like author, email, or description.

### Manifest Example

```json
{
  "name": "MyToken",
  "groups": [],
  "features": {},
  "supportedstandards": ["NEP-17"],
  "abi": {
    "methods": [
      {
        "name": "balanceOf",
        "parameters": [{ "name": "account", "type": "Hash160" }],
        "returntype": "Integer",
        "offset": 45,
        "safe": true
      }
    ],
    "events": []
  },
  "permissions": [
    {
      "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
      "methods": ["transfer"]
    }
  ]
}
```
