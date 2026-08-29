# General Security Considerations

While Solidity itself provides various security guarantees, writing smart contracts requires a deep understanding of the underlying execution environment. Contracts are completely public, meaning their source code and compiled bytecode can be inspected by anyone, and anyone can interact with them.

::: tip 💡 NeoVM Difference
The most critical security differences between EVM and NeoVM lie in **Authorization (Witnesses vs `msg.sender`)** and **Contract Permissions (Manifests)**.
:::

## Pitfalls

### Authorization: Witnesses over `msg.sender`

On Ethereum, authorization is implicit through `msg.sender`. The caller's address is automatically set by the EVM to the contract or account that initiated the specific `CALL`.

On Neo, authorization is explicit. The transaction includes cryptographic witnesses (signatures) that prove the caller controls specific addresses. 

While Neo DevPack for Solidity maps `msg.sender` to `Runtime.getCallingScriptHash()` (which securely returns the hash of the contract calling your contract), **it is an anti-pattern to use this for user authorization**. You must use `Runtime.checkWitness()`.

```solidity
// ❌ EVM pattern — relies on msg.sender implicitly
function withdraw(uint256 amount) public {
    require(msg.sender == owner, "not owner");
    // ...
}

// ✅ Neo-idiomatic — explicit witness check
import {Runtime} from "@neo/Runtime.sol";

function withdraw(uint256 amount) public {
    // Cryptographically verifies the transaction was signed by `owner`
    require(Runtime.checkWitness(owner), "not authorized");
    // ...
}
```
If you rely on `msg.sender == user`, your contract is vulnerable to users who wrap their calls in another contract (because `msg.sender` will become the intermediate contract, not the user). `Runtime.checkWitness()` verifies the user authorized the transaction globally, regardless of the call depth.

### Manifest Permissions and Wildcards

Neo N3 enforces contract call permissions at the VM level via the contract's manifest. If your contract attempts to call a target contract or method not explicitly allowed in your manifest, execution will fault.

By default, dynamic calls in Solidity (like `address(target).call(...)`) force the compiler to emit wildcard permissions (`"contract": "*", "methods": "*"`). **This is a severe security risk**, as it allows your contract to be hijacked to interact with malicious contracts, effectively bypassing the platform's permission sandbox.

### Checked Arithmetic and Wraparound

NeoVM uses arbitrary-precision `BigInteger` representation internally, but Neo DevPack for Solidity emits Solidity 0.8 fixed-width overflow and underflow guards outside `unchecked` blocks. Checked overflows revert with `Panic(0x11)`.

If your contract's security model relies on wraparound, put the operation in an `unchecked` block and cover that behavior with tests. `unchecked` does not suppress division-by-zero or modulo-by-zero panics.

### Upgradeability

Ethereum relies on `delegatecall` and complex proxy architectures to make contracts upgradeable. NeoVM inherently blocks `delegatecall` because it isolates storage context strictly per-contract.

However, Neo natively supports upgrading contracts using `ContractManagement.update()`. This replaces the executing code while retaining the existing storage. 

## Recommendations

### 1. Compile with Strict Permissions
Always compile production contracts with strict wildcard denial flags:
```bash
neo-solc MyContract.sol --deny-wildcard-contracts --deny-wildcard-methods
```
If dynamic routing is strictly required by your architecture, use NatSpec overrides (`@custom:neo.manifest.permissions`) to explicitly allowlist specific trusted hashes rather than defaulting to `*`.

### 2. Guard the Upgrade Method
Because `ContractManagement.update()` is a native intrinsic that modifies the code of the contract, ensure that access to your update method is heavily protected. It should ideally be behind a multisig or DAO governance check, relying on `Runtime.checkWitness()`.

### 3. Handle NEP-17 Payments Carefully
When implementing `onNEP17Payment` (the equivalent to Ethereum's `receive()`), always verify the `msg.sender` to ensure the token being received is the one you expect (e.g., ensuring `msg.sender == NativeContracts.GAS`). Otherwise, malicious users can send worthless tokens to spoof deposits.
