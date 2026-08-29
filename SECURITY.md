# Security Policy

## Supported Versions

The Neo DevPack for Solidity is currently in active development. We recommend using the latest stable release for production use.

| Version | Supported        |
| ------- | ---------------- |
| 0.18.x  | ✅ Supported     |
| 0.17.x  | ⚠️ Best-effort   |
| < 0.17  | ❌ Not supported |

## Reporting a Vulnerability

We take the security of the Neo DevPack for Solidity seriously. If you believe you have found a security vulnerability, please report it responsibly.

### Reporting Process

1. **Do NOT** open a public GitHub issue for security vulnerabilities
2. Email your report to: **security@r3e.network**
3. Include a detailed description of the vulnerability
4. Include steps to reproduce the issue (if possible)
5. Include any relevant code snippets or logs

### What to Expect

- We will acknowledge your report within 48 hours
- We will provide an initial assessment within 7 days
- We will keep you informed of the progress
- We will not disclose the vulnerability publicly until a fix is released

## Security Best Practices

### For Users

#### 1. Use Specific Manifest Permissions

Avoid wildcard contract permissions in production:

```bash
# ❌ Avoid (wildcard)
neo-solc contract.sol -o build/

# ✅ Recommended (explicit permissions)
neo-solc contract.sol --manifest-permissions permissions.json -o build/
```

Create a `permissions.json` file:

```json
{
  "permissions": [
    {
      "contract": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
      "methods": ["transfer", "balanceOf"]
    }
  ]
}
```

#### 2. Use Strict Compilation Flags

```bash
# Deny wildcard permissions
neo-solc contract.sol --deny-wildcard-permissions -o build/

# Even stricter: deny wildcard contracts and methods
neo-solc contract.sol --deny-wildcard-contracts --deny-wildcard-methods -o build/
```

#### 3. Test on TestNet First

Always deploy and test on Neo N3 TestNet before mainnet deployment:

```bash
# Deploy to TestNet
neo-cli contract deploy build/contract.nef build/contract.manifest.json --network testnet
```

#### 4. Verify Compilation Output

Check the generated manifest for excessive permissions:

```bash
cat build/contract.manifest.json | jq '.permissions'
```

### For Developers

#### 1. Code Review Requirements

- All code changes require at least one reviewer
- Security-sensitive changes require two reviewers
- Use `cargo clippy` and `cargo fmt` before submitting

#### 2. Dependency Management

- Use `cargo audit` to check for vulnerable dependencies
- Keep dependencies up to date
- Review dependency changes in PRs

#### 3. Testing Requirements

- Add unit tests for new functionality
- Add integration tests for security-sensitive code
- Run `make check-coverage` to ensure test coverage

## Known Security Considerations

### Smart Contract Security

When compiling Solidity contracts for Neo N3, be aware of:

1. **Reentrancy**: Use reentrancy guards for functions that transfer value
2. **Integer Overflow**: Solidity 0.8+ includes overflow protection
3. **Access Control**: Verify visibility modifiers and role-based access
4. **Gas Limits**: Neo N3 has different gas semantics than EVM

### Compiler Security

The compiler handles untrusted input (Solidity source code):

- Parser is protected against malformed input
- Type checking prevents invalid operations
- Memory safety ensured by Rust's ownership model
- Unbounded recursion is prevented

## Links

- [Neo N3 Documentation](https://docs.neo.org/)
- [Neo N3 Security Best Practices](https://docs.neo.org/docs/en-us/basic/environment.html)
- [Solidity Security Considerations](https://docs.soliditylang.org/en/latest/security-considerations.html)

---

Thank you for helping keep the Neo DevPack for Solidity and the Neo ecosystem secure.
