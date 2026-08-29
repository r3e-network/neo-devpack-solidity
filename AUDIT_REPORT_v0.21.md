# Neo DevPack for Solidity — 系统性审计报告 (v0.21.0)

> **审计日期**: 2026-06-17
> **审计范围**: Solidity→NeoVM 编译器 + 运行时模拟器 + devpack 标准库 + 测试/CI
> **代码规模**: 485 Rust 文件 (~84K LOC) + 35 .sol 文件 + 109 Rust 测试文件
> **审计方法**: 5 个并行子代理 + 关键结论独立二次验证（直接读代码核对）
> **审计基准**: 当前 `main` 分支 (commit 17664cd)；不依赖任何过时文档

---

## 0. 一句话结论

**编译器内核（IR lowering、opcode 映射、NEF/manifest 生成、devpack NEP 标准）整体是正确的、可达生产水准的**；v0.20→v0.21 的 "deep correctness pass" 确实落地了一批真正的等价性修复（uint256 软件除模、位宽正确的 `~`、`unchecked{}`、负数符号扩展等）。**但运行时模拟器（`src/runtime/`）作为"NeoVM 等价物"存在多个高危偏差**，且测试金字塔恰好把唯一一个真正能抓住这些偏差的 oracle（28 个 Neo-Express 真链脚本）排除在 CI 之外。这意味着：**"在模拟器里跑通的合约"和"在 Neo N3 主网跑通的合约"之间目前没有强保证**。

---

## 1. 总体架构与判断

```
Solidity 源 → solang-parser → 元数据提取 → IR (类Yul) → 优化器 → NeoVM 字节码 → NEF/Manifest
                                                    ↓
                                    内嵌 NeoRuntime 模拟器（用于测试）
```

架构方向**正确**：不是逐 EVM opcode 翻译（不可行），而是在 IR 层做语义等价转换。这是这类跨 VM 编译器唯一正确的策略。v0.21 的"deep correctness pass"反映出团队已经意识到并系统处理了"BigInteger VM 模拟定宽 EVM 语义"这一核心难题。

**按子系统的健康度分级：**

| 子系统 | 健康度 | 一句话 |
|---|---|---|
| Frontend / 元数据提取 | 🟢 良好 | `receive`/`fallback` 重映射有死角；其余扎实 |
| IR lowering / 类型系统 | 🟢 良好 | 软件除模/位宽/短路求值都正确；`mulmod/addmod` 有残留 |
| 字节码发射 + 优化器 | 🟡 中等 | 优化 pass 谨慎且分层；存在 3 处未检查的 slice 写 |
| NEF / manifest / 权限 | 🟢 良好 | checksum/token 表/权限推导符合 Neo 规范 |
| **内嵌 NeoRuntime 模拟器** | 🔴 **多个高危** | serialize/gas/CheckSig/multisig 账号/BLS/CallFlags 全部偏离真链 |
| devpack NEP 标准 | 🟢 良好 | 签名合规；NEP-11 自托管 + NEP-24 tokenId 两个 bug |
| 测试 / fuzz / CI | 🟡 中等 | 内部一致性极强；但缺真链 oracle 兜底 |

---

## 2. 🔴 严重问题（Correctness / Security）

### S1. 模拟器的 `StdLib.serialize` 输出 JSON，而非 Neo 二进制格式
**`src/runtime/execution/execution_impl_part2_native/stdlib.rs:323-330`** — 已二次核对

```rust
"serialize" => { ... let bytes = serde_json::to_vec(&value).unwrap_or_default(); ... }
"deserialize" => { ... serde_json::from_slice::<StackItem>(&bytes).unwrap_or(Null) }
```

Neo N3 的 `StdLib.serialize` 产生的是 **`Neo.IO.Json.SmartContract` / BinarySerializer 二进制**，这里用的是 `serde_json`。后果：合约的 `serialize → 存储写入 → 读取 → deserialize` 在模拟器里 round-trip 成功（因为两端都用 JSON），**但上链后字节数/长度/哈希全不同**。如果序列化结果被当作 storage key、或被 `keccak256` 做指纹、或与其它原生合约互操作，会**静默错位**。`Find` 的 `DESERIALIZE_VALUES`（`storage_ops.rs:69-71`）也把存储里的二进制当 JSON 解析。

### S2. 存储类 syscall 的 gas 偏低约 1000×
**`src/runtime/spec/gas.rs:15` + `src/runtime/execution/instruction/syscall.rs:10`** — 已二次核对

```rust
const STORAGE_PUT_PER_BYTE_GAS: u64 = 100;
"System.Storage.Put" => 1_000,   // 平价 base
```

Neo N3 主网 `Policy.storagePrice` 默认 **100,000/byte**（`0.001 GAS/byte`）。模拟器 1KB 写入 ≈ 100K gas；主网 ≈ 100M gas。后果：本地 gas 预算能过的合约，**部署到主网会 gas 耗尽**；任何 `require(gasleft() > X)` 类断言在两端表现完全不同。

### S3. `System.Crypto.CheckSig/CheckMultisig` 校验的是一个合成哈希，不是交易的签名哈希
**`src/runtime/execution/helpers/crypto.rs:104-126`** — 已二次核对

```rust
let bytecode_hash = Sha256::digest(&self.bytecode);   // ← 不是 tx hash
hasher_input.extend_from_slice(&bytecode_hash);
hasher_input.extend_from_slice(account...);
hasher_input.extend_from_slice(&invocation_counter.to_le_bytes());
Sha256::digest(&hasher_input)
```

Neo N3 校验签名是对**脚本容器的可验证交易摘要**做的。这里用的是 `SHA256(bytecode || account || invCounter)` —— 一个与任何真实签名载荷无关的合成值。**所有 CheckSig/CheckMultisig 的结果对真实签名都没有意义。** 测试文件 `runtime_syscall_tests.rs:150-216` 自己承认这点（只断言 `result == [0] || [1]`）。任何依赖 `CheckSig`（不是 `CheckWitness`）做权限的合约，模拟器结果不可信。目前**没有注入真实签名哈希的 API**。

### S4. `System.Contract.CreateMultisigAccount` 算出的 UInt160 是错的
**`src/runtime/execution/syscalls/contract.rs:43-53`**

```rust
input.extend_from_slice(&m); input.extend_from_slice(&pubkeys);
let digest = Sha256::digest(&input);
self.push_stack(StackItem::byte_array(digest[0..20].to_vec()));  // ← 错
```

Neo N3 是构造 multisig 验证脚本（`PUSH m / ... / SYSCALL CheckMultisig`）再做 `RIPEMD160(SHA256(script))`。这里直接 `SHA256(m||pubkeys)[0..20]` —— 没有 RIPEMD160、没有脚本构造。每个 multisig 派生地址都与链上不同。`CreateStandardAccount`（紧邻）是对的，所以这是单独遗漏。（注：与其上方的 secp256k1 单签 `CreateStandardAccount` 正确形成对比。）

### S5. BLS12-381 `pairing` 的 Gt 输出用 `Debug` 格式化作为线上编码
**`src/runtime/execution/execution_impl_part2_native/crypto.rs:307-317`**

```rust
fn bls_serialize_gt(gt: &bls12_381::Gt) -> Vec<u8> {
    format!("{gt:?}").into_bytes()   // ← Debug，非规范编码
}
```

代码注释承认这只够"两次模拟器调用相互比对"的差分测试用。没有可互操作的 Gt 序列化。任何消费 pairing 结果的合约（BLS 签名验证、zk-SNARK）拿到的是一个不可移植的字节团。

### S6. CALLT / native 调用绕过 `CallFlags` 与 manifest 权限检查
**`src/runtime/execution/instruction/flow/calls.rs:120-126` + `execution_impl_part2_contract_call.rs:10`**

```rust
let _flags = self.pop_stack()?; // call flags 在模拟器中被忽略
...
let result = self.invoke_native_contract(&token.hash, &token.method, params);
```

Neo N3 会按调用强制 `CallFlags`（ReadStates/WriteStates/AllowCall/AllowNotify），不匹配即 fault；`GetCallFlags` 被硬编码成 `0x0F`（`contract.rs:11-12`）。后果：以 `staticcall` 语义发起的调用（本应拒绝写）在模拟器里能写；manifest 声明的方法/权限检查也被跳过。**测试通过，上链 fault**。

### S7. 存储层在内部调用 revert 时不会回滚到该调用帧的快照
**`bridge/bridge_impl_core/execute.rs:48` + `helpers/storage_ops.rs:277` + `try_frames.rs:30-39`**

`drain_dirty_storage_overlay` 只在顶层 halt 时一次性提交；`unbind_storage()` 出错时全清。但同一顶层执行内部，如果 `System.Contract.Call` 进入一个 self-offsets 子帧、子帧 `Storage.Put` 后通过 `THROW` revert，THROW 会展开 `call_stack` 并截断 eval stack —— **但失败子帧的脏存储写仍然留在 `storage_overlay` 里，在顶层 halt 时被提交**。真 Neo N3 会在任何内部异常时把存储回滚到该调用的快照。这对"对外部调用做 try/catch 且外部调用写存储"的合约是真实的正确性偏差。

---

## 3. 🟡 中等问题（值得修，不是阻断）

### M-IR. `mulmod` / `addmod` 绕过 uint256 软件除模路径
**`src/ir/expressions/calls/variable_calls.rs:129-135`**

直接发 `BinaryOp(Mul|Add|Mod)`，不走 `emit_u256_divmod_ir`。对 `uint256` 且模数/积 ≥ 2^255，原生 NeoVM `MOD` 是有符号的，余数会错；`mulmod` 的中间积 `a*b` 还可能在窄路径上静默回绕。EVM 正确做法是复用 uint256 软件除模和 limb-mul。这是 IR 层唯一一个实质性 EVM 偏差。

### M-FE1. `receive()` 与 `onNEP17Payment` 共存时成为静默死代码
**`src/solidity/convert/functions.rs:9-31`** + 集成测试 `receive_hooks.rs:44-86`

同时定义两者时，`receive()` 被保留为一个名为 `"receive"` 的 Neo 方法，而 Neo 入金只调 `onNEP17Payment`。`receive()` 体内 `if (msg.value > 0)` 永远为 false（msg.value 仅在 onNEP17Payment 里有值），整个 body 永不执行。只有 W105 一个泛化警告，严重低估风险。**建议：要么 strip，要么硬错。**

### M-FE2. `fallback()` 走通用路径，EVM catch-all 语义丢失且仅警告
**`src/solidity/convert/functions.rs`（无 `FunctionTy::Fallback` 分支）**

Neo 没有未知方法分发到名为 `fallback` 的方法的机制。代理/中继类合约依赖 `fallback()` 的语义会静默失效。仅 W105。**建议：给 fallback 一个显式 convert 分支 + 大字警告或硬错。**

### M-FE3. `has_explicit_on_nep17_payment` 检测大小写敏感
**`src/solidity/convert/contract.rs:18-20`** 用 `== "onNEP17Payment"`，而 NEP 模式校验（`erc_nep_patterns.rs:175`）用 `eq_ignore_ascii_case`。`function onnep17payment(...)` 不会被识别为显式回调，`receive()` 会被重映射出第二个 `onNEP17Payment`，造成 manifest 名称冲突。

### M-FE4. `ETHER_UNIT_RE` 把合法变量名/注释误判为硬错
**`src/solidity/upgrade.rs:68`** — `Regex::new(r"\b(?:wei|gwei|szabo|finney|ether)\b")`，对源码裸文本扫描，不剥离注释、不识别标识符上下文。`uint ether = msg.value;` 或 `// whether` 里的 "ether" 会被升级成 `UpgradeSeverity::Error`（`upgrade.rs:179`）。DEX 代码里很常见。

### M-FE5. 公共同名状态变量跨继承的类型冲突只警告不报错
**`src/solidity/analyse/inheritance/flatten.rs:114`** + `state_variables.rs:94-106`。`state_variables.extend` 只按名字去重，同名不同类型的公共状态变量会产生 W122 警告但**共享存储槽**（slot 由名字派生）。硬冲突检查只在 sibling-merge 路径，不在 inheritance-flatten 路径。

### M-IR2. 逻辑或/与的右操作数未强制 bool 规整
**`src/ir/statements/logical.rs:231-285`**。短路走到右分支时把右表达式原值留在栈上不做 bool 转换。Solidity 类型保证是 bool，但 Yul/assembly 注入的非 bool 值或前端类型推断失误时会泄漏。低现实风险但隐性。

### M-IR3. 同符号不同位宽的整型在重载解析中被认为匹配
**`src/ir/context/lowering_context.rs:252-260` `overload_arg_matches`** 把 `uint8` 和 `uint256` 当匹配。Solidity 禁止这种歧义重载，前端应先拒；但 IR 解析器没防御性区分位宽。

### M-BC1. 字节码发射有 3 处未检查的 slice 写 / 静默写零
- **`src/cli/bytecode/bytecode_emit_ir.rs:379,386`** — jump 落空解析为 `local.len()`（静默 fall-through），`local[position..position+4]` 无边界检查；若 `prune_after_terminator` 删了 label 但留下 jump（它只匹配 `Label` 不处理孤儿 `Jump` 操作数），`copy_from_slice` 会 panic。
- **`src/cli/bytecode/bytecode_core.rs:206`** — 未解析调用目标只 `eprintln!` 然后留零字节；零 CALL_L 偏移 = 上链死循环/自调用。应是 `Err`。
- **`src/cli/bytecode/bytecode_core.rs:264-271`** `apply_method_tokens` 静默 `continue` 跳过越界 patch，留下 `00 00` = token #0 = 错的 native 调用。

### M-BC2. `x == true → x` 这类恒等优化对非 bool 输入改变可观测行为
**`src/cli/ir_optimize/neovm.rs:142-152`**（-O3）。NeoVM 下 `(5 == true)` 应为 `false`，但优化成裸 `x` 留下 `5`。下游期待 0/1 bool 的消费方会拿到错值。

### M-BC3. `MethodToken::serialize` 用 `assert!` 而非 `Result`
**`src/neo/method_token.rs:51`**。虽然 `build_nef_with_tokens` 先校验，但 `serialize` 是 `pub(super)`，任何直接调用方（测试/其它 neo 模块）会 panic 而非返回 Err。

### M-DEV1. NEP-11 `_transfer`/`_mint` 缺 `to != address(this)` 自托管短路
**`devpack/standards/NEP11.sol:597-603, 629`**。NEP-17 在 `NEP17.sol:478` 有这个短路，NEP-11 没有。NFT 合约把 token 转给自己（托管模式）会触发 `INEP11Receiver(address(this)).onNEP11Payment(...)`，合约通常不实现该 selector → try 失败 → `revert NEP11InvalidReceiver` → **所有 NFT 自托管流被硬阻断**。与 NEP-17 设计不一致。

### M-DEV2. NEP-24 `royaltyInfo` 用 `bytes32 tokenId`，而 NEP-11/NEP-26 用动态 `bytes`
**`devpack/standards/NEP24.sol:27,83`** vs `NEP11.sol`/`NEP26.sol:17`。tokenId 非 32 字节的 NFT 无法无损传给版税查询。

### M-DEV3. NEP-17 授权混合了 NEP-17 witness 与 ERC-20 allowance
**`devpack/standards/NEP17.sol:261-265`** `require(ownerAuthorized || _allowances[from][msg.sender] >= amount, ...)`。一个有任意非零 allowance 的 spender 可以不经 owner witness 转账 —— 比 ERC-20 严（多了 witness 选项）但又允许纯 allowance 路径绕过 NEP-17 的 checkWitness 期望。是有意的混合语义，但应在文档里写清。

### M-RT1. `System.Runtime.GetNotifications` 永远返回空数组
**`src/runtime/execution/syscalls/runtime.rs:306-310`**。任何消费自己 emit 的通知的合约在模拟器里拿不到。

### M-RT2. `System.Runtime.GetRandom` 是确定性的 `SHA256(block||counter)`
**`src/runtime/execution/syscalls/runtime.rs:111-134`**。同一高度对每个合约调用都一样、跨运行也一样。真链用 VRF 派生的 `nextNonce`。依赖 GetRandom 做"唯一性"的测试无意义。

### M-RT3. `System.Runtime.CheckWitness` 语义偏离
**`src/runtime/execution/syscalls/runtime.rs:277-305`**。`witness_signers` 为空时回退到比较 `caller_account`/`default_account_bytes`。真 Neo 校验的是"脚本容器带着一个 witness，其验证脚本哈希等于给定 hash"。除默认账号外的 signer 行为不一致。

### M-RT4. revert-vs-fault 靠子串 `"THROW"` 在渲染文本里判别
**`src/runtime/bridge/bridge_impl_core/execute.rs:143-148`** `if rendered.contains("THROW") { RevertExecution } else { Fault }`。任何消息文本里含 "THROW" 的 fault（含用户 `revert "THROW"`）会被误分类。

### M-TEST1. `e2e_compilation_tests.rs` 只编译不执行
~60 个测试只跑 `neo-solc` + 断言 exit 0 + nef/manifest 存在 + magic 字节。**没有任何一个把产物字节码跑过运行时。** 编译出坏字节码也能过。

### M-TEST2. `conformance/` 对的是内部一致性，不是规范
`infrastructure.rs:72-141` 在内嵌 NeoRuntime 里跑，对比 `vectors.rs` 里手写的期望 i64。没有 EVM 参考、没有 Neo N3 节点参考、没有 neo-vm 官方 `Tests/Engine/*.json`。名字夸大了它证明的东西。

### M-TEST3. 优化器差分只覆盖 pure；storage/events 只比 return_data
`optimizer_props.rs` O0↔O3 差分很强，但仅限 pure 算术/位运算。`optimizer_semantic_equivalence_storage_and_events`（case 6）只比 `return_data`，**不比 emit 的 Notify payload、不比最终存储态**。一个把 `PUT` 和 `Notify` 顺序重排的优化 bug（链上可观测，此断言看不见）会通过。

---

## 4. 🟢 低风险 / 改进建议（精选）

- **L-FE1** `frontend_parse.rs:94` `parse_source` 用 `_ => {}` 静默丢弃未识别的 `SourceUnitPart` 变体 —— 未来 Solidity 新顶层构造会无声编译成空。建议发"unsupported top-level construct"警告。
- **L-FE2** W121 诊断码被两个无关诊断复用（`state_variables.rs:120` 与 `library.rs:63`），用户按码过滤无法区分。
- **L-BC** CALLT token index `u16::try_from(index).unwrap_or(u16::MAX)`（`bytecode_core.rs:252`）—— 当前 `MAX_METHOD_TOKENS=512` 下安全，但 cap 一旦上调就会别名到 #511。脆耦合，应改 `Err`。
- **L-RT** `recoverSecp256K1` 接受 `v ∈ {0,1,27..=30}` 慷慨但合理。`murmur3_32`（`helpers/crypto.rs`）实现正确。`Storage.Put` 的 `MaxStorageKeySize=64` / `MaxStorageValueSize=65535` 校验（`storage.rs:123-140`）符合 Neo N3 共识。`FindOptions` 校验（`storage_ops.rs:31-54`）正确镜像 C# 节点的互斥规则。
- **L-DEV** `CompleteNEP17Token.getGovernanceInfo()`（`:709`）对 `abi.encode("proposal")` 前缀迭代，但提案存在 Solidity keccak 槽里，迭代器永远空 → `activeProposals`/`executedProposals` 恒为 0。

---

## 5. 测试 / CI 专项（这是最重要的结构性风险）

| 项 | 现状 | 风险 |
|---|---|---|
| 内部一致性 / 纯算术差分 | 🟢 极强（594 个 proptest 位点，44 文件，O0↔O3 差分，4KB 哈希差分） | 编译器不会崩，算术/位运算语义可信 |
| 哈希 / murmur / itoa 差分 | 🟢 强（对 sha2/sha3/ripemd/murmur3 crate） | 加密接线可信 |
| 编译崩溃 fuzz | 🟢 强（fuzz_target_1 跑 O0..O3 + catch_unwind；runtime_exec 有 gas/memory 上限） | DoS 向量覆盖好 |
| `e2e_compilation_tests` | 🟡 仅编译不执行 | 编译出坏字节码也能过 |
| Gas 断言 | 🟡 只比模拟器自己的近似表（`spec/gas.rs` 自述"approximate"） | 给"gas 已测"的错觉 |
| CheckSig/CheckMultisig | 🔴 零正面正确性断言（测试自己承认 hash 不匹配） | 签名验证路径无保障 |
| BLS12-381 / secp256k1-recover 差分 | 🔴 故意不断言（runtime 返 Null/空） | ZK 验证类合约零保障 |
| **Neo-Express 真链脚本（28 个）** | 🔴 **存在但完全不在 CI** | 唯一能抓"模拟器过、链上挂"的 oracle 被排除在外 |

**CI gates**：`ci.yml` 跑 `fmt --check` / `clippy -D warnings` / `cargo test --workspace` / release build + 一个只检查 ERC20 NEF magic 和 manifest 字段的 trivial smoke。`fuzz.yml` 每日跑 proptest（100 cases）+ 4 个 cargo-fuzz 目标 60 秒。`security.yml` 是 **report-only 不 gate**。

**最高优先级行动**：把 `make test-deploy-smoke-full`（28 个 Neo-Express 脚本，真正部署到 Neo-Express 链并断言 `vmstate=HALT`、返回值、`msg.sender`、存储 round-trip）加进 CI 的必过门。这是当前唯一能抓 S1–S7 那一类 bug 的地方，现在却完全裸奔。

---

## 6. 优点（要保留的）

1. **架构方向正确** —— IR 层语义等价转换是这类跨 VM 编译器唯一可行策略。
2. **v0.21 "deep correctness pass" 是真的** —— uint256 软件除模（Hacker's Delight divmod）、`x^2^255` 无符号比较、逻辑 SHR、位宽正确的 `~x`（按位宽截断）、窄整型 checked-arith 范围检查（有符号/无符号双域）、`intN.min/-1` 除法溢出、`intN.min` 一元负 guard、后缀自增自减回绕恢复、负数多值返回符号扩展 —— 都是 conformant 的。`mulmod/addmod` 是唯一漏网的。
3. **优化器分层谨慎** —— `-O2` 只做有 4096-bit 字面量上限的常量折叠；危险重写（peephole/bool-opt）都在 `-O3`。默认构建安全。
4. **NEF/manifest 发射符合 Neo N3 规范** —— checksum = `SHA256(SHA256(prefix))[..4]` LE；method-token 格式（20B hash ‖ varbytes(method) ‖ u16 params ‖ u8 has_return ‖ u8 flags）；`MAX_METHOD_TOKENS=512`；interop ID = `SHA256(name)[..4]`。
5. **存储键派生正确** —— `storage_key.rs` 刻意把 mapping 槽哈希留到链上代码做（`keccak256(serialize(key)||slot)`），避免了离线"错误派生"的静默键错配；状态变量用 `SHA256(name)` 256 位哈希，无跨变量碰撞风险。
6. **devpack NEP 签名合规** —— NEP-17 四参 `transfer(address,address,uint256,Any)`、`onNEP17Payment(address,uint256,Any)`、`decimals returns uint8`；NEP-11 `bytes` tokenId（1..64 字节校验）；NEP-26 也用 `bytes`。`Syscalls.sol` 里所有 `"System.*"` 字面量与编译器 syscall registry **逐字一致**（已交叉核对）。
7. **fuzz/prop 基础设施扎实** —— 594 proptest 位点、O0↔O3 四级差分、4KB+块边界哈希差分、`catch_unwind` 崩溃 fuzz、NEF/disasm/manifest roundtrip fuzz 带真实种子语料。
8. **输入路径基本无裸 panic** —— IR/statements 的 `unwrap`/`expect`/`unreachable!` 都在编译器内部不变量上（Yul op match 臂、预分配的临时局部），不在用户可控数据上。前端 `parse_solidity_guarded` 把 parser panic 转成诊断并限制栈深，确实稳健。

---

## 7. 推荐的修复优先级

### P0（阻断生产部署可信度，1–2 周内）
1. **S3 + S4** —— CheckSig 用真实签名哈希；`CreateMultisigAccount` 走脚本构造 + RIPEMD160。提供注入签名哈希的 API。这两项不修，任何签名/多签权限合约在模拟器里都不可信。
2. **CI 接入 Neo-Express 真链 smoke**（M-TEST1/2/3 的根因解药）—— 把 `make test-deploy-smoke-full` 设为必过门。这一步会自动暴露 S1/S2/S6/S7 在真链上的表现。
3. **S1** —— `StdLib.serialize/deserialize` 改用 Neo 二进制格式（或至少让 `serialize` 与 `jsonSerialize` 区分开，并让存储 round-trip 测试用二进制）。

### P1（正确性，1 个月内）
4. **S2** —— 把 gas 表对齐 Neo N3 主网（至少 `Storage.Put` 按 `storagePrice` per-byte；`Notify/Log` 按载荷大小）。
5. **S6 + S7** —— CallFlags 强制 + 内部调用 revert 时存储回滚到帧快照。
6. **M-IR** —— `mulmod/addmod` 走 uint256 软件除模路径。
7. **M-FE1 + M-FE2** —— `receive()`/`fallback()` 共存与 catch-all 语义：要么 strip/硬错，要么显式 convert + 大字警告。
8. **M-DEV1** —— NEP-11 加 `to != address(this)` 自托管短路，与 NEP-17 对齐。

### P2（健壮性 / 打磨，择期）
9. **M-BC1** —— 3 处未检查 slice 写改 `Result`；未解析调用目标从 `eprintln!` 升级为 `Err`。
10. **M-BC2 + M-BC3** —— `x==true→x` 恒等优化加 bool 范围守卫；`MethodToken::serialize` 改 `Result`。
11. **M-FE3/4/5** —— onNEP17Payment 大小写不敏感；`ETHER_UNIT_RE` 注释/标识符感知；继承 flatten 路径补类型冲突硬检查。
12. **M-RT1/2/3/4** —— GetNotifications 返真实通知；GetRandom 至少注入可配种子；CheckWitness 支持 witness 列表注入；revert/fault 判别不用子串。
13. **M-DEV2/3** —— NEP-24 tokenId 改 `bytes`；NEP-17 授权混合语义写进文档。
14. **L-*** —— 诊断码去重、CALLT index 越界改 Err、parse_source 静默丢弃改警告等。

---

## 8. 总评

**编译器（frontend → IR → bytecode → NEF/manifest + devpack）**：生产就绪级别。核心映射正确，v0.21 的等价性修复扎实，devpack NEP 标准合规。剩余 IR/前端/字节码问题都是边界情况和打磨级别，不会导致资金损失。

**内嵌 NeoRuntime 模拟器**：**结构性偏乐观**。它在"结构上像 NeoVM"（栈、槽、TRY/ENDTRY 展开、ISTYPE/CONVERT、FIND 迭代成型）层面是忠实的，但在**原生/syscall 表面**层面充满 stub 和近似：serialize 格式错、签名验签用合成哈希、multisig 账号编码错、BLS Gt 不可移植、存储/CheckSig gas 偏低 ~1000×、CallFlags 与权限检查被跳过、内部 revert 不回滚存储。**任何完全在模拟器内测试的合约，相当一部分会在真链上静默错位或 fault。**

**最关键的一句话**：把 28 个 Neo-Express 真链脚本接进 CI 必过门，是用最小工作量闭合"模拟器过、链上挂"这一最危险回归类的唯一且最高杠杆的动作。在做任何新功能之前，应该先做这件事。
