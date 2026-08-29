# NatSpec Format

Solidity contracts can use a special form of comments to provide rich documentation for functions, return variables and more. This special form is named the Ethereum Natural Language Specification Format (NatSpec).

Neo DevPack for Solidity fully parses NatSpec comments and embeds standard descriptions into the generated manifest where appropriate.

## Neo-Specific NatSpec Directives

::: tip 💡 NeoVM Difference
In addition to standard documentation tags like `@title`, `@notice`, and `@param`, Neo DevPack for Solidity uses the `@custom:` prefix to control Neo-specific compilation mechanics, such as manifest permissions.
:::

### `@custom:neo.manifest.permissions`

This directive allows you to explicitly define the `permissions` array in your compiled `.manifest.json`. This is crucial when the compiler is forced to fall back to wildcards due to dynamic `address.call()` usage, but you want to enforce strict compilation policies.

```solidity
/// @notice A contract that interacts with dynamic decentralized exchanges
/// @custom:neo.manifest.permissions [{"contract": "0x1234...", "methods": ["swap"]}, {"contract": "0xabcd...", "methods": ["*"]}]
contract MyDEXRouter {
    function route(address pool, bytes memory data) public {
        (bool success, ) = pool.call(data);
        require(success, "swap failed");
    }
}
```

When compiled with `--manifest-permissions-mode replace-wildcards`, the compiler will swap its inherently insecure wildcard inference with the strict array you provided in the NatSpec comment.

### `@custom:neo.manifest.trusts`

The `trusts` property in a Neo manifest indicates which other contracts are allowed to call this contract without triggering user warnings in wallet interfaces. You can define this directly in the source:

```solidity
/// @custom:neo.manifest.trusts ["0x1111222233334444555566667777888899990000"]
contract TrustedChild {
    // ...
}
```

### Supported Tags

The compiler supports and extracts the following standard NatSpec tags:

| Tag | Context | Description |
| --- | --- | --- |
| `@title` | Contract | A title that should describe the contract/interface. |
| `@notice` | Contract, Function, Event | Explain to an end user what this does. |
| `@dev` | Contract, Function, Event | Explain to a developer any extra details. |
| `@param` | Function, Event | Documents a parameter. |
| `@return` | Function | Documents the return variable of a contract’s function. |