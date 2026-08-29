# Solidity Support Matrix

> **Compiler**: neo-devpack-solidity v0.27.0
> **Parser**: foundry-solang-parser 0.3.9
> **Target**: NeoVM (Neo N3, node v3.10.0)
> **Solidity versions**: 0.5.x–0.8.x accepted (0.8.x primary/best-supported)
> **Audit date**: 2026-07-02

Legend:

- ✅ Fully supported
- ⚠️ Partial support (see notes)
- ❌ Not supported
- 🚫 Intentionally blocked with diagnostic error

Scope note: this matrix describes neo-solc support on NeoVM, not full EVM
compatibility. The compiler accepts Solidity 0.5.x–0.8.x source (0.8.x is the
primary, best-supported line). Features marked supported may still have
Neo-specific semantics. For migration guidance and production validation gaps,
see `docs/internals/parity-and-limitations.md`.

---

## A. Types

| Feature                                  | Status | Notes                                                                 |
| ---------------------------------------- | ------ | --------------------------------------------------------------------- |
| `bool`                                   | ✅     | Maps to NeoVM Boolean                                                 |
| `int8`..`int256`                         | ✅     | All widths parsed; NeoVM uses arbitrary-precision BigInteger          |
| `uint8`..`uint256`                       | ✅     | All widths parsed; NeoVM uses arbitrary-precision BigInteger          |
| `address`                                | ✅     | Maps to Neo UInt160 (Hash160, 20 bytes)                               |
| `address payable`                        | ⚠️     | Parsed and canonicalized to `address`; `.transfer()` / `.send()` map to Neo GAS transfer semantics, not EVM attached value |
| `bytes1`..`bytes32`                      | ✅     | Fixed-length byte arrays via `NeoType::ByteArray { fixed_len }`       |
| `bytes` (dynamic)                        | ✅     | Dynamic byte array                                                    |
| `string`                                 | ✅     | UTF-8 string type                                                     |
| `enum`                                   | ✅     | Converted via `convert_enum`; backed by uint8                         |
| `struct`                                 | ✅     | Full struct support with nested fields; `StructDefinition` converted  |
| `mapping(K => V)`                        | ✅     | Storage mappings with Neo StorageMap; key type validation enforced    |
| `T[]` (dynamic array)                    | ✅     | `new T[](n)` allocation supported                                     |
| `T[N]` (fixed array)                     | ✅     | Parsed; `new T[N]` supported when `N` is compile-time constant; `.length` returns the declared bound `N` (including storage arrays) |
| `fixed` / `ufixed`                       | ❌     | Not supported (also unsupported in mainline Solidity)                 |
| User-defined value types (`type X is Y`) | ✅     | Transparent type aliases; `wrap`/`unwrap` compile to no-ops           |
| `bytes.concat(...)`                      | ✅     | Chains NeoVM CAT opcodes; zero args produce empty byte array          |
| `string.concat(...)`                     | ✅     | Same implementation as `bytes.concat` via CAT opcode chain            |
| Contract types (e.g., `IERC20`)          | ✅     | Resolved to Neo UInt160 address; interface types tracked              |
| Tuple types                              | ✅     | Represented as NeoVM arrays internally                                |
| Function types (`function(...) internal / external`) | ❌     | Not representable on NeoVM; state variables, locals, params, and return types of function type are rejected by the `NeoType` resolver. Use named functions and inheritance instead of function pointers. |

---

## B. Expressions

| Feature                                       | Status | Notes                                                                                                           |
| --------------------------------------------- | ------ | --------------------------------------------------------------------------------------------------------------- |
| Arithmetic (`+`, `-`, `*`, `/`, `%`)          | ✅     | Binary ops via `try_lower_expression_binary_ops`                                                                |
| Comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`) | ✅     | Via `try_lower_expression_comparisons`                                                                          |
| Logical (`&&`, `\|\|`, `!`)                   | ✅     | Short-circuit evaluation in `logical.rs`                                                                        |
| Bitwise (`&`, `\|`, `^`, `~`, `<<`, `>>`)     | ✅     | Full bitwise support; `bytesN` `<<` / `>>` shift the big-endian face value with EVM truncation semantics (all widths). Known gap: `bytesN` bitwise compound-assign (`\|=` etc.) stores byte-reversed; reads are correct |
| Unary (`++`, `--`, `-`, `!`)                  | ✅     | Pre/post increment/decrement                                                                                    |
| Ternary (`? :`)                               | ✅     | `ConditionalOperator` lowered with labels                                                                       |
| Assignment (`=`, `+=`, `-=`, etc.)            | ✅     | Compound assignments in `assignments/compound.rs`                                                               |
| `delete`                                      | ✅     | State vars, mapping entries, locals, array elements, struct fields                                              |
| Tuple expressions `(a, b, c)`                 | ✅     | `Expression::List` lowered to NeoVM arrays                                                                      |
| Tuple destructuring `(a, b) = f()`            | ⚠️     | Nested destructuring assignment supported; some complex target forms still require intermediate locals          |
| Type casting                                  | ✅     | `TypeCastingShowcase.sol` example compiles                                                                      |
| `type(X).min` / `type(X).max`                 | ✅     | Supported for integer types                                                                                     |
| `type(T).name`                                | ✅     | Compile-time string constant for contract/type names                                                            |
| `type(I).interfaceId`                         | ✅     | Computed from selector XOR of interface methods                                                                 |
| `abi.encode(...)`                             | ⚠️     | Supported for selected compiler/runtime paths; dynamic payloads and standalone production use need Neo-Express validation |
| `abi.encodePacked(...)`                       | ⚠️     | Same as `abi.encode` — used for Neo contract call encoding                                                      |
| `abi.encodeWithSignature(...)`                | ⚠️     | In low-level call contexts it rewrites to Neo contract calls; standalone use approximates calldata as selector plus encoded args |
| `abi.encodeWithSelector(...)`                 | ⚠️     | In low-level call contexts it rewrites to Neo contract calls; standalone use approximates calldata as selector plus encoded args |
| `abi.encodeCall(...)`                         | ✅     | Maps to `StdLib.serialize`                                                                                     |
| `abi.decode(...)`                             | ✅     | Maps to `StdLib.deserialize`; type tuple parsed from second argument                                            |
| Named function call args `f({x: 1})`          | ✅     | Named args reordered to positional order at IR level                                                            |

---

## C. Statements

| Feature                   | Status | Notes                                                                                                       |
| ------------------------- | ------ | ----------------------------------------------------------------------------------------------------------- |
| `if` / `else`             | ✅     | Standard conditional branching                                                                              |
| `for` loop                | ✅     | Init, condition, post, body all lowered                                                                     |
| `while` loop              | ✅     | Condition + body                                                                                            |
| `do...while` loop         | ✅     | Body + condition                                                                                            |
| `break`                   | ✅     | Loop break                                                                                                  |
| `continue`                | ✅     | Loop continue                                                                                               |
| `return`                  | ✅     | Single and multi-value returns                                                                              |
| `emit Event(...)`         | ✅     | Maps to `Runtime.Notify`; indexed params supported                                                          |
| `revert(...)`             | ✅     | Maps to NeoVM `ABORT` with message                                                                          |
| `revert CustomError(...)` | ✅     | Named revert with args; `RevertNamedArgs` also handled                                                      |
| Variable declaration      | ✅     | Local variable definitions with optional initializer                                                        |
| Block `{ ... }`           | ✅     | Scoped statement blocks                                                                                     |
| `unchecked { ... }`       | ✅     | Suppresses Solidity 0.8 checked-arithmetic guards inside the block; fixed-width arithmetic wraps, including `int256` add/sub/mul two's-complement wrap mod 2^256 (`type(int256).max + 1 == type(int256).min`). Checked mode still panics (0x11) |
| `assembly { ... }`        | ⚠️     | Limited Yul subset lowering; unsupported EVM-only operations warn and emit no assembly logic for that block |
| `try` / `catch`           | ✅     | Maps to NeoVM TRY/ENDTRY; single catch clause preferred                                                     |
| `catch Error(string)`     | ✅     | Named catch with parameter binding                                                                          |
| `catch Panic(uint256)`    | ✅     | Matches the canonical `keccak256("Panic(uint256)")[..4] = 0x4e487b71` selector on the revert envelope and decodes the 32-byte BE code (Task #103) |
| `catch (bytes)`           | ✅     | Low-level catch with raw bytes                                                                              |

---

## D. Functions

| Feature                          | Status | Notes                                                                                                                                           |
| -------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| Regular functions                | ✅     | Public, external, internal, private                                                                                                             |
| Free functions (file-scope)      | ✅     | Functions declared outside any contract are callable; overloads deduplicated by name+arity, and all overloads are callable                      |
| Constructor                      | ✅     | Single constructor; multiple constructors rejected                                                                                              |
| `view` / `pure`                  | ✅     | State mutability tracked and enforced at IR level                                                                                               |
| `payable`                        | ⚠️     | Parsed; `payable` on non-receive functions warns (Neo has no native gas payment)                                                                |
| `returns (T)`                    | ✅     | Single return type                                                                                                                              |
| `returns (T1, T2, ...)`          | ✅     | Multi-return via NeoVM arrays                                                                                                                   |
| Function overloading             | ⚠️     | Supported with `neo_name` mangling; one canonical ABI name is preserved and non-primary overloads are exported under generated Neo method names |
| `modifier`                       | ✅     | Full modifier expansion with `_` placeholder substitution                                                                                       |
| `receive()`                      | ⚠️     | **Silently remapped** to `onNEP17Payment(address,uint256,bytes)` in the manifest when no explicit `onNEP17Payment` is declared (NEP-17 convention, see `src/solidity/convert/functions.rs:32`). Ethereum devs: the body is preserved but exported under a new ABI name. See "receive()/fallback() remapping" note below. |
| `fallback()`                     | ⚠️     | Retains the name `fallback` in the manifest. No EVM-style unknown-method dispatch; compiler emits diagnostic W105 suggesting `onNEP17Payment()`.  |
| `virtual` / `override`           | ✅     | Inheritance flattening resolves overrides; multi-level chains supported                                                                         |
| Function selectors (`.selector`) | ✅     | Computed from canonical parameter types                                                                                                         |
| NatSpec comments                 | ✅     | `@notice`, `@dev`, `@param`, `@return` preserved in metadata                                                                                    |

---

## E. OOP Features

| Feature              | Status | Notes                                                                                                                                                    |
| -------------------- | ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Single inheritance   | ✅     | C3 linearization with `flatten_contract_inheritance`                                                                                                     |
| Multiple inheritance | ✅     | Diamond inheritance detected; constructor arg conflicts reported                                                                                         |
| `interface`          | ✅     | Interface types tracked; methods validated                                                                                                               |
| `abstract contract`  | ✅     | Fully validated; unimplemented functions detected; non-abstract contracts get actionable errors. A concrete contract holding an abstract-contract-typed field is not forced to implement that field's virtuals |
| `library`            | ⚠️     | Builtin devpack libraries are compiler intrinsics; user-defined libraries are merged/inlined, but still cannot model deployable library state or linking |
| `using X for Y`      | ✅     | Library member-call syntax fully supported; `using X for *` and `using {f,g} for T` included                                                             |
| User-defined operators (`using {f as +} for T`) | ⚠️     | Solidity 0.8.19 operator binding compiles with warning `W_USER_DEFINED_OPERATOR`, but operators are NOT dispatched to the bound function — raw arithmetic runs on the underlying representation. Call the bound function explicitly |
| `super` keyword      | ✅     | Supported via inheritance flattening with `__super_` method preservation                                                                                 |
| `is` (inheritance)   | ✅     | Inheritance specifiers fully processed                                                                                                                   |
| Constructor chaining | ✅     | Base constructor arguments resolved from inheritance specifiers                                                                                          |
| Event inheritance    | ✅     | Interface events collected recursively via `collect_interface_events_recursive`; a sibling's INHERITED (interface/base) events are carried through the sibling-merge and via cross-contract `new` |
| Library-declared events | ✅  | Events declared inside a `library` are inlined into the consuming contract's ABI when emitted                                                            |

---

## F. Storage & Memory

| Feature                          | Status | Notes                                                                 |
| -------------------------------- | ------ | --------------------------------------------------------------------- |
| State variables                  | ✅     | Mapped to Neo Storage with prefix-based keys                          |
| `constant`                       | ✅     | Compile-time constants inlined                                        |
| `immutable`                      | ✅     | Tracked via `is_immutable` flag; modification blocked at compile time |
| `transient` state variables      | ⚠️     | Solidity 0.8.28 (EIP-1153) `transient` compiles to regular PERSISTENT storage with warning `W_TRANSIENT_PERSISTED` (NeoVM has no transient store); values are NOT cleared at transaction end — clear manually at every entry point |
| `memory` keyword                 | ✅     | Parsed; NeoVM is stack-based so memory is implicit                    |
| `storage` keyword                | ✅     | Storage references for mappings and state variables                   |
| `calldata` keyword               | ✅     | Parsed; treated as `memory` (correct for NeoVM — no calldata region)  |
| Nested mappings                  | ✅     | `mapping(K1 => mapping(K2 => V))` with composite storage keys         |
| Struct in storage                | ✅     | Serialized/deserialized via `StdLib.serialize`/`StdLib.deserialize`   |
| Array `.push()` / `.pop()`       | ✅     | Storage array operations supported                                    |
| Array `.length`                  | ✅     | Both memory and storage arrays                                        |
| `new bytes(n)` / `new string(n)` | ✅     | Buffer allocation via `NEWBUFFER`                                     |
| `new T[](n)`                     | ✅     | Dynamic array allocation via `NEWARRAY`                               |
| `new Contract(...)`              | ⚠️     | Does not deploy a child contract; constructor-like logic is inlined/simulated and a zero-address placeholder is produced. Use `ContractManagement.deploy(...)` for real deployment |
| `new X{salt: s}()` (CREATE2)     | ⚠️     | Salted creation is accepted (parses and compiles); the `salt` is ignored because Neo N3 has no CREATE2. Same inline/simulate model as `new Contract(...)` |

---

## G. Error Handling

| Feature                                   | Status | Notes                                                                                                              |
| ----------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------ |
| `require(condition)`                      | ✅     | Maps to NeoVM ASSERT                                                                                               |
| `require(condition, "msg")`               | ✅     | ASSERT with message                                                                                                |
| `require(condition, CustomError(...))`    | ✅     | Error name and arg count preserved in NeoVM THROW message                                                          |
| `assert(condition)`                       | ✅     | Maps to NeoVM ASSERT                                                                                               |
| `revert()`                                | ✅     | Maps to NeoVM ABORT                                                                                                |
| `revert("message")`                       | ✅     | ABORT with message                                                                                                 |
| `revert CustomError(...)`                 | ✅     | Named revert with arguments                                                                                        |
| Custom error definitions (`error X(...)`) | ✅     | Parsed and used in revert statements                                                                               |
| `try` / `catch`                           | ✅     | NeoVM TRY/ENDTRY structured exception handling                                                                     |
| `try` with return binding                 | ✅     | `try f() returns (uint r) { ... }` supported                                                                       |
| Multiple catch clauses                    | ✅     | Lowered with EVM-canonical selector guards for `Error(string)` and `Panic(uint256)`; user-defined named catches retain legacy guards |

---

## H. EVM-Specific Features (with Neo Alternatives)

| Feature                                 | Status | Neo Alternative                                                                                                                     |
| --------------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------- |
| `msg.sender`                            | ✅     | Maps to `Runtime.GetCallingScriptHash()`; internal/self-offset runtime paths may inject a direct-caller override                    |
| `msg.value`                             | ⚠️     | Only mapped inside `onNEP17Payment` callback                                                                                        |
| `msg.data`                              | ⚠️     | Approximated as selector plus encoded current args outside onNEP17Payment                                                            |
| `msg.sig`                               | ⚠️     | Approximated as the current function selector; internal-call propagation still differs from EVM                                     |
| `block.timestamp`                       | ✅     | Maps to `Runtime.GetTime()` (normalized to seconds)                                                                                 |
| `block.number`                          | ✅     | Maps to `Ledger.CurrentIndex()`                                                                                                     |
| `block.chainid`                         | ✅     | Maps to Neo network magic number                                                                                                    |
| `block.coinbase`                        | ✅     | Auto-mapped to `address(0)` with warning (dBFT has no miner)                                                                        |
| `block.difficulty` / `block.prevrandao` | ✅     | Auto-mapped to `Runtime.getRandom()` with warning                                                                                   |
| `block.gaslimit`                        | ✅     | Auto-mapped to `Policy.getExecFeeFactor()` with warning                                                                             |
| `block.basefee`                         | ✅     | Auto-mapped to `Policy.getFeePerByte()` with warning                                                                                |
| `tx.origin`                             | ⚠️     | Parsed; warning about authorization risks                                                                                           |
| `tx.gasprice`                           | ✅     | Auto-mapped to `Policy.getFeePerByte()` with warning                                                                                |
| `gasleft()`                             | ✅     | Auto-mapped to `System.Runtime.GasLeft` syscall                                                                                     |
| `blockhash(n)`                          | ✅     | Auto-mapped to `Ledger.getBlockHash()` with warning                                                                                 |
| `block.sha3`                            | ⚠️     | Auto-mapped to `Ledger.currentHash`; deprecated in Solidity 0.8+                                                                    |
| `keccak256(...)`                        | ✅     | Maps to `CryptoLib.keccak256`                                                                                                       |
| `sha256(...)`                           | ✅     | Maps to `CryptoLib.sha256`                                                                                                          |
| `ecrecover(...)`                        | ✅     | Uses `CryptoLib.recoverSecp256K1`, then derives the Ethereum address with `keccak256` and `RIGHT 20`                                |
| `selfdestruct(addr)`                    | ✅     | Auto-mapped to `ContractManagement.destroy()` with warning                                                                          |
| `address.call(...)`                     | ⚠️     | Maps to `System.Contract.Call`; return wrapping and ABI payload parity differ from EVM and need Neo-Express validation              |
| `address.staticcall(...)`               | ⚠️     | Maps to `System.Contract.Call` (read-only flag); return wrapping and ABI payload parity differ from EVM                             |
| `address.delegatecall(...)` / `callcode(...)` | 🚫     | Blocked at compile time; Neo N3 has no equivalent caller-storage execution semantics                                                |
| `address.transfer(amount)`              | ✅     | Auto-mapped to `GAS.transfer(from,to,amount,data)`; aborts on transfer fail                                                         |
| `address.send(amount)`                  | ✅     | Auto-mapped to `GAS.transfer(from,to,amount,data)`; returns bool                                                                    |
| `address.balance`                       | ✅     | Auto-mapped to `GAS.balanceOf(address)`                                                                                             |
| `address.code`                          | ⚠️     | Returns Neo contract script bytes; non-contracts return empty bytes. `address.code.length` still maps to a contract-existence check |
| `address.codehash`                      | ✅     | Auto-mapped to contract script hash with warning; non-contract → bytes32(0)                                                         |
| Ether units (`wei`, `gwei`, `ether`)    | ⚠️     | Parsed; warning that Neo uses GAS token (10^8 decimals)                                                                             |
| Time units (`seconds`, `minutes`, etc.) | ✅     | Compile-time constants (normalized to seconds)                                                                                      |
| `this` keyword                          | ✅     | Maps to `Runtime.GetExecutingScriptHash()`; correct Neo equivalent                                                                  |
| `type(X).creationCode`                  | ⚠️     | Compiler-emitted deterministic NEF3-shaped payload for hashing compatibility; not EVM bytecode and requires an in-graph type        |
| `type(X).runtimeCode`                   | ⚠️     | Same deterministic Neo-shaped payload model as `creationCode`; not production bytecode introspection                                |

---

## I. ERC to NEP Protocol Mapping

| ERC Standard                 | NEP Equivalent                | Status | Notes                                                                         |
| ---------------------------- | ----------------------------- | ------ | ----------------------------------------------------------------------------- |
| ERC-20 (Fungible Token)      | NEP-17                        | ✅     | Auto-detected; `transfer(to,amount)` warns to use 4-param NEP-17 form         |
| ERC-721 (NFT)                | NEP-11                        | ✅     | Auto-detected; `transferFrom` warns to use NEP-11 `transfer(to,tokenId,data)` |
| ERC-20 `approve`/`allowance` | N/A                           | ⚠️     | Warning: not part of NEP-17 spec; Neo uses `Runtime.checkWitness()`           |
| ERC-165 `supportsInterface`  | Manifest `supportedstandards` | ⚠️     | Warning: unnecessary on Neo; manifest-based discovery                         |
| ERC-4626 (Tokenized Vault)   | NEP-17                        | ⚠️     | Vault logic compiles; ERC-20 interactions must use NEP-17 equivalents         |
| ERC-2981 (Royalty)           | NEP-24                        | ✅     | Auto-detected; multiple royalty recipients supported                          |
| `receive()` / `fallback()`   | `onNEP17Payment()`            | ⚠️     | `receive()` silently remapped; `fallback()` kept as-is. See note below.       |

### receive()/fallback() remapping

Neo N3 uses the `onNEP17Payment(address from, uint256 amount, bytes data)` callback rather than EVM-style `receive()` / `fallback()` dispatch. To keep Solidity source portable:

- A Solidity `receive() external payable { ... }` with no explicit `onNEP17Payment` in the contract is **silently remapped** in the manifest to `onNEP17Payment(address,uint256,bytes)`. The body is preserved verbatim; the ABI name, parameters, and selector are rewritten (see `src/solidity/convert/functions.rs:32`).
- If the contract **already declares** an explicit `onNEP17Payment`, `receive()` keeps the name `receive` in the manifest (no remap), and diagnostic `W105` flags it as having no effect on Neo N3.
- `fallback()` is never remapped and keeps its Solidity name in the manifest; there is no Neo equivalent to EVM fallback dispatch, so diagnostic `W105` suggests `onNEP17Payment` instead.

**Migration guidance for Ethereum developers**: if your contract needs to accept tokens, prefer declaring `onNEP17Payment(address from, uint256 amount, Any data)` directly or inherit the devpack [EVM Compatibility Layer](/additional-material/neo-devpack/evm-compatibility-layer). It surfaces the sender, amount, token contract, and attached data that Neo's NEP-17 transfer provides. Use `receive()` only if you are porting EVM source and accept that tooling will see the entrypoint under the Neo NEP-17 name.

---

## Summary

| Category            | ✅      | ⚠️     | ❌    | 🚫    |
| ------------------- | ------- | ------ | ----- | ----- |
| A. Types            | 17      | 1      | 2     | 0     |
| B. Expressions      | 16      | 5      | 0     | 0     |
| C. Statements       | 17      | 1      | 0     | 0     |
| D. Functions        | 10      | 4      | 0     | 0     |
| E. OOP Features     | 10      | 2      | 0     | 0     |
| F. Storage & Memory | 12      | 3      | 0     | 0     |
| G. Error Handling   | 11      | 0      | 0     | 0     |
| H. EVM-Specific     | 21      | 11     | 0     | 1     |
| I. ERC-NEP Mapping  | 3       | 4      | 0     | 0     |
| **Total**           | **117** | **31** | **2** | **1** |

**Total features audited: 151**

- ✅ Fully supported: 117 (77%)
- ⚠️ Partial support: 31 (21%)
- ❌ Not supported: 2 (1%)
- 🚫 Intentionally blocked: 1 (1%)
