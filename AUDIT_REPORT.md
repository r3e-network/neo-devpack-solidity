# Neo DevPack for Solidity 编译器审计报告 (历史快照)

> **历史快照说明**: 本文件保留 v0.12.0 (2025-07-14) 的中文审计报告，仅作为历史
> 参考。当前编译器版本为 v0.18.0，最新的功能矩阵、运行时状态、以及限制说明请参考：
>
> - `docs/SOLIDITY_SUPPORT_MATRIX.md`
> - `docs/internals/parity-and-limitations.md`
> - `CHANGELOG.md`
> - `README.md`
>
> 本报告中的版本号、行数、缺陷状态已不反映当前代码库；仅用于追溯当时的设计判断。

**版本**: v0.12.0  
**审计日期**: 2025-07-14  
**审计范围**: Solidity 语法支持完整性、EVM→NeoVM 映射正确性、类型系统、全局变量映射  
**代码规模**: 532 Rust 文件, ~57K LOC

---

## 1. 总体评价

编译器架构清晰，6 阶段 pipeline 设计合理：

```
solang-parser → Solidity Metadata → IR → Optimizer → NeoVM Bytecode → NEF/Manifest
```

核心映射思路正确：不是逐 EVM opcode 翻译，而是在 IR 层做语义等价转换。这是正确的设计决策——EVM 和 NeoVM 是根本不同的栈机器，逐 opcode 翻译不可行。

---

## 2. Solidity 语法支持覆盖

### ✅ 已支持（正确）

| 特性                                          | 状态 | 备注                                       |
| --------------------------------------------- | ---- | ------------------------------------------ |
| 合约/接口/库/抽象合约                         | ✅   | ContractKind 四种全覆盖                    |
| 继承 (is)                                     | ✅   | C3 线性化 + 虚函数/override                |
| 构造函数                                      | ✅   | FunctionKind::Constructor                  |
| 状态变量 (constant/immutable)                 | ✅   | 含 storage key 生成                        |
| 函数可见性 (public/external/internal/private) | ✅   |                                            |
| 状态可变性 (pure/view/payable/nonpayable)     | ✅   | view/pure 自动降级为 ReadOnly CallFlags    |
| 修饰符 (modifier)                             | ✅   | 完整的 modifier 展开和 `_` 替换            |
| 事件 (emit)                                   | ✅   | → System.Runtime.Notify                    |
| require/assert                                | ✅   | → NeoVM THROW                              |
| revert (含自定义错误)                         | ✅   | 含 RevertNamedArgs                         |
| if/else/while/do-while/for                    | ✅   | 完整控制流                                 |
| break/continue                                | ✅   | 含循环标签栈                               |
| try/catch                                     | ✅   | → NeoVM TRY_L/ENDTRY_L，含多 catch 子句    |
| 三元运算符 (?:)                               | ✅   | 短路求值正确                               |
| 逻辑 &&/\|\|                                  | ✅   | 短路求值正确                               |
| 复合赋值 (+=, -=, \*=, etc.)                  | ✅   | 含 mapping/struct/array 元素               |
| 前/后缀自增自减 (++/--)                       | ✅   | 语义正确（后缀返回原值）                   |
| 幂运算 (\*\*)                                 | ✅   | 常量折叠 + 运行时快速幂算法                |
| 数组 (动态/固定)                              | ✅   | 含 push/pop/length                         |
| 数组切片                                      | ✅   |                                            |
| 数组字面量                                    | ✅   |                                            |
| Mapping                                       | ✅   | 含嵌套 mapping                             |
| 结构体                                        | ✅   | 含嵌套结构体、存储布局                     |
| 枚举                                          | ✅   | → uint8                                    |
| 用户自定义值类型 (type X is Y)                | ✅   | wrap/unwrap 编译为 no-op                   |
| using X for Y                                 | ✅   | 库函数合并到合约                           |
| Natspec 文档                                  | ✅   | @title/@notice/@dev/@param/@return/@custom |
| 字符串/bytes 操作                             | ✅   | concat, length, 切片                       |
| 类型转换                                      | ✅   | 含 address ↔ bytes20                       |
| type(X).max/min                               | ✅   | 含 uint/int 全位宽                         |
| type(I).interfaceId                           | ✅   | ERC-165 兼容                               |
| type(C).name                                  | ✅   |                                            |
| unchecked { }                                 | ✅   | NeoVM BigInteger 无溢出，语义正确的 no-op  |
| 多返回值                                      | ✅   | → NeoVM Array 打包                         |
| 命名返回变量                                  | ✅   | 含隐式 return                              |
| 变量遮蔽检测                                  | ✅   |                                            |
| ERC→NEP 模式检测                              | ✅   | 14+ 种模式检查                             |

### ⚠️ 兼容性映射（有 warning，语义近似）

| 特性                                 | 映射方式                                  | 风险等级                |
| ------------------------------------ | ----------------------------------------- | ----------------------- | ----------------- | --------------------- |
| msg.sender                           | → System.Runtime.GetCallingScriptHash     | ✅ 正确                 |
| msg.value                            | → onNEP17Payment 参数 / 0                 | ⚠️ 语义不同             |
| msg.data                             | → `selector                               |                         | abi.encode(args)` | ⚠️ 近似映射（回调外） |
| msg.sig                              | → 当前函数 selector                       | ⚠️ 内部调用语义不同     |
| tx.origin                            | → System.Runtime.GetCallingScriptHash     | ⚠️ 有 warning，语义不同 |
| tx.gasprice                          | → Policy.getFeePerByte()                  | ⚠️ 近似映射             |
| tx.hash                              | → System.Runtime.GetScriptContainer       | ⚠️ 返回类型不同         |
| block.timestamp                      | → System.Runtime.GetTime                  | ✅ 正确                 |
| block.number                         | → Ledger.currentIndex                     | ✅ 正确                 |
| block.chainid                        | → System.Runtime.GetNetwork               | ✅ 合理映射             |
| block.coinbase                       | → address(0)                              | ⚠️ dBFT 无矿工          |
| block.difficulty/prevrandao          | → System.Runtime.GetRandom                | ⚠️ 近似映射             |
| block.gaslimit                       | → Policy.getExecFeeFactor()               | ⚠️ 近似映射             |
| block.basefee                        | → Policy.getFeePerByte()                  | ⚠️ 近似映射             |
| address.balance                      | → GAS.balanceOf(addr)                     | ✅ 合理映射             |
| address.code.length                  | → ContractManagement.isContract() ? 1 : 0 | ✅ 巧妙映射             |
| address.codehash                     | → 合约地址=scriptHash, 非合约=bytes32(0)  | ✅ 合理                 |
| address.transfer(amount)             | → GAS.transfer() + abort on fail          | ✅ 正确语义             |
| address.send(amount)                 | → GAS.transfer() 返回 bool                | ✅ 正确语义             |
| address.call/staticcall/delegatecall | → System.Contract.Call + TRY/ENDTRY       | ✅ 专业实现             |
| abi.encode/encodePacked              | → StdLib.serialize                        | ✅                      |
| abi.decode                           | → StdLib.deserialize                      | ✅                      |
| abi.encodeWithSignature              | → 解析方法名 + Contract.Call              | ✅ 巧妙                 |
| abi.encodeWithSelector               | → 解析 selector → 方法名                  | ✅                      |
| abi.encodeCall                       | → 解析函数引用 → 方法名                   | ✅                      |
| keccak256                            | → CryptoLib.keccak256                     | ✅                      |
| ecrecover                            | → CryptoLib.recoverSecp256K1              | ✅                      |
| ether 单位 (wei/gwei/ether)          | → 编译 warning                            | ✅ 正确处理             |

### ❌ 未支持 / 需要注意

| 特性                  | 状态    | 影响                                         |
| --------------------- | ------- | -------------------------------------------- |
| 内联汇编 (assembly)   | ⚠️ 部分 | 特殊 handler (extsload/exttload)，其余 no-op |
| fixed/ufixed (定点数) | ❌      | TypeParseError::FixedPoint，NeoVM 不支持     |
| receive()/fallback()  | ⚠️      | 检测并建议用 onNEP17Payment                  |
| selfdestruct          | ❌      | Neo 无等价操作                               |
| create/create2        | ❌      | Neo 用 ContractManagement.deploy             |
| blockhash(n)          | ❌      | 需要 Ledger.getBlock(n)                      |
| gasleft()             | ❌      | Neo 无等价操作                               |
| 多重继承钻石问题      | ⚠️      | C3 线性化处理，但复杂场景可能有边界情况      |

---

## 3. NeoVM Opcode 映射正确性

### 3.1 算术运算

| Solidity | IR            | NeoVM Opcode | 正确性 |
| -------- | ------------- | ------------ | ------ |
| +        | BinaryOp::Add | 0x9E (ADD)   | ✅     |
| -        | BinaryOp::Sub | 0x9F (SUB)   | ✅     |
| \*       | BinaryOp::Mul | 0xA0 (MUL)   | ✅     |
| /        | BinaryOp::Div | 0xA1 (DIV)   | ✅     |
| %        | BinaryOp::Mod | 0xA2 (MOD)   | ✅     |
| \*\*     | 快速幂循环    | MUL+SHR 组合 | ✅     |
| - (一元) | × (-1)        | PUSHM1 + MUL | ✅     |

**注意**: NeoVM 使用 BigInteger，无溢出问题。Solidity 的 `unchecked {}` 正确编译为 no-op。

### 3.2 比较运算

| Solidity | NeoVM Opcode    | 正确性 |
| -------- | --------------- | ------ |
| <        | 0xB5 (LT)       | ✅     |
| <=       | 0xB6 (LE)       | ✅     |
| >        | 0xB7 (GT)       | ✅     |
| >=       | 0xB8 (GE)       | ✅     |
| ==       | 0x97 (EQUAL)    | ✅     |
| !=       | 0x98 (NOTEQUAL) | ✅     |

### 3.3 位运算

| Solidity | NeoVM Opcode  | 正确性 |
| -------- | ------------- | ------ |
| &        | 0x91 (AND)    | ✅     |
| \|       | 0x92 (OR)     | ✅     |
| ^        | 0x93 (XOR)    | ✅     |
| ~        | 0x90 (INVERT) | ✅     |
| <<       | 0xA8 (SHL)    | ✅     |
| >>       | 0xA9 (SHR)    | ✅     |

### 3.4 控制流

| Solidity       | NeoVM Opcode                     | 正确性 |
| -------------- | -------------------------------- | ------ |
| if/else        | JMP_L (0x23) + JMPIFNOT_L (0x27) | ✅     |
| while          | JMP_L + JMPIFNOT_L               | ✅     |
| for            | JMP_L + JMPIFNOT_L               | ✅     |
| return         | RET (0x40)                       | ✅     |
| revert/require | THROW (0x3A)                     | ✅     |
| assert         | THROW + "Panic: 0x01"            | ✅     |
| try/catch      | TRY_L (0x3C) + ENDTRY_L (0x3E)   | ✅     |
| break          | JMP_L → end_label                | ✅     |
| continue       | JMP_L → post_label               | ✅     |

### 3.5 函数调用

| 操作         | NeoVM 实现                           | 正确性 |
| ------------ | ------------------------------------ | ------ |
| 内部调用     | CALL_L (0x35) + REVERSEN 参数反转    | ✅     |
| 外部调用     | System.Contract.Call syscall         | ✅     |
| 原生合约调用 | CALLT (0x37) 或 System.Contract.Call | ✅     |
| INITSLOT     | 0x57 + local_count + arg_count       | ✅     |

**关键细节**: 参数反转逻辑正确——NeoVM INITSLOT 从栈顶开始分配参数，而 Solidity 从左到右求值，所以调用前需要 REVERSEN。

### 3.6 存储操作

| 操作       | NeoVM 实现               | 正确性 |
| ---------- | ------------------------ | ------ |
| 状态变量读 | System.Storage.Get       | ✅     |
| 状态变量写 | System.Storage.Put       | ✅     |
| Mapping 读 | 键哈希 + Storage.Get     | ✅     |
| Mapping 写 | 键哈希 + Storage.Put     | ✅     |
| 结构体字段 | 复合键 + Storage.Get/Put | ✅     |
| 数组元素   | 索引键 + Storage.Get/Put | ✅     |
| delete     | System.Storage.Delete    | ✅     |

### 3.7 整数字面量编码

| 范围    | NeoVM 指令               | 正确性             |
| ------- | ------------------------ | ------------------ |
| -1      | PUSHM1 (0x0F)            | ✅                 |
| 0       | PUSH0 (0x10)             | ✅                 |
| 1-16    | PUSH1-PUSH16 (0x11-0x20) | ✅                 |
| i8      | PUSHINT8 (0x00)          | ✅                 |
| i16     | PUSHINT16 (0x01)         | ✅                 |
| i32     | PUSHINT32 (0x02)         | ✅                 |
| i64     | PUSHINT64 (0x03)         | ✅                 |
| i128    | PUSHINT128 (0x04)        | ✅                 |
| i256    | PUSHINT256 (0x05)        | ✅                 |
| >256bit | PUSHDATA + PUSH0 + ADD   | ✅ 巧妙的 fallback |

**注意**: uint256.max (2^256-1) 无法用 PUSHINT256 直接表示（有符号），使用 PUSHDATA+ADD 强制转换为 Integer 类型。这是正确的处理方式。

---

## 4. 发现的问题

### 4.1 🔴 严重问题

**无严重问题发现。** 核心编译路径正确。

### 4.2 🟡 中等问题

#### P1: Solidity 有符号整数右移语义差异

Solidity 的 `>>` 对有符号整数执行算术右移（SAR），对无符号整数执行逻辑右移（SHR）。当前 IR 只有一个 `BinaryOperator::Shr`，映射到 NeoVM 的 `SHR` (0xA9)。

NeoVM 的 `SHR` 对 BigInteger 执行的是算术右移（保留符号位），所以对有符号整数是正确的。但对无符号整数，如果值的高位为 1（在 256 位表示中），行为可能不同。

**实际风险**: 低。NeoVM BigInteger 没有固定位宽，负数右移保留符号，正数右移等价于逻辑右移。只有当 uint256 值 >= 2^255 时才可能有差异，但 NeoVM 会将其视为正数（因为 BigInteger 无固定位宽）。

**建议**: 在文档中明确说明此行为差异。

#### P2: 有符号除法/取模语义

Solidity 的有符号除法向零截断（与 C 语言一致），NeoVM 的 DIV (0xA1) 也是向零截断。✅ 一致。

但 Solidity 的有符号取模遵循 `a % b` 的符号与 `a` 一致，NeoVM 的 MOD (0xA2) 也是如此。✅ 一致。

#### P3: 低级调用的 delegatecall 映射

`address.delegatecall()` 在 EVM 中使用调用者的存储上下文执行被调用合约的代码。Neo N3 没有等价概念。当前实现将其映射为普通的 `System.Contract.Call`，这在语义上是不同的。

**建议**: 对 delegatecall 发出更强的 warning 或编译错误，因为语义差异可能导致安全问题。

#### P4: msg.value 在非 onNEP17Payment 上下文中

在非 `onNEP17Payment` 函数中访问 `msg.value` 会加载 `RuntimeValue::MsgValue`，但 Neo N3 没有"附带 value 的调用"概念。当前实现会返回什么值取决于 runtime 的 `LoadRuntimeValue` 实现。

**建议**: 在非 onNEP17Payment 上下文中访问 msg.value 时发出 warning。

### 4.3 🟢 低风险 / 改进建议

#### P5: 内联汇编的静默 no-op

未识别的 `assembly {}` 块被静默编译为 no-op。虽然有特殊 handler 处理 extsload/exttload，但大多数 EVM 汇编指令（如 mload/mstore/sload/sstore）会被静默忽略。

**建议**: 对包含实际指令的 assembly 块发出 warning。

#### P6: 事件参数的 indexed 属性

Solidity 事件的 `indexed` 参数在 EVM 中作为 topic 存储。Neo 的 `System.Runtime.Notify` 没有 topic 概念，所有参数都在 state 数组中。

**当前处理**: EventParameter 记录了 `indexed` 属性但在代码生成时未区分处理。这是正确的——Neo 不需要区分。

#### P7: 函数重载的 Neo 名称冲突

Solidity 允许函数重载（同名不同参数），但 Neo ABI 按名称+参数数量分发。当前实现使用 `neo_name` 字段做名称修饰（mangling）来避免冲突。

**建议**: 确认名称修饰策略在所有边界情况下都能正确工作（如多个重载有相同参数数量但不同类型）。

---

## 5. 原生合约映射完整性

### 5.1 已映射的原生合约

| 原生合约           | 方法覆盖                                               | 状态    |
| ------------------ | ------------------------------------------------------ | ------- |
| NeoToken           | transfer, vote, getCandidates, balanceOf, etc.         | ✅ 完整 |
| GasToken           | transfer, balanceOf, symbol, decimals, totalSupply     | ✅ 完整 |
| ContractManagement | deploy, update, destroy, getContract, isContract, etc. | ✅ 完整 |
| Policy             | getFeePerByte, getExecFeeFactor, getStoragePrice, etc. | ✅ 完整 |
| Oracle             | request, getPrice, verify                              | ✅      |
| RoleManagement     | designateAsRole, getDesignatedByRole                   | ✅      |
| Notary             | balanceOf, lockDepositUntil, withdraw, etc.            | ✅      |
| Treasury           | verify, onNEP17Payment, onNEP11Payment                 | ✅      |
| Ledger             | currentIndex, getBlock, getTransaction, etc.           | ✅ 完整 |
| CryptoLib          | sha256, ripemd160, keccak256, verifyWithECDsa, etc.    | ✅ 完整 |
| StdLib             | serialize, deserialize, base64/58, json, etc.          | ✅ 完整 |

### 5.2 CallFlags 处理

- 只读方法 → `CallFlags.ReadOnly` (0x05)
- 写入方法 → `CallFlags.All` (0x0F)
- view/pure 函数中的外部调用 → 自动降级为 ReadOnly

✅ 正确且专业。

### 5.3 CALLT 优化

对符合条件的原生合约调用使用 `CALLT` (0x37) 指令而非 `System.Contract.Call` syscall，减少字节码大小和 GAS 消耗。

✅ 专业优化。

---

## 6. NEF/Manifest 生成

- NEF magic: `4e454633` (NEF3) ✅
- Manifest: name, abi, permissions, supportedstandards ✅
- 权限推导: 从代码中使用的 syscall/native call 自动推导 ✅
- 标准检测: NEP-17, NEP-11 自动检测 ✅

---

## 7. 优化器评估

### 当前优化 passes (O3):

1. **常量折叠** — 含嵌套表达式、identity ops、强度削减 ✅
2. **死代码消除** — leave/return 后的代码 ✅
3. **函数内联** — 小函数内联 ✅
4. **CSE** — 公共子表达式消除 ✅
5. **NeoVM peephole** — PUSH+DROP、StoreLocal+LoadLocal→DUP+StoreLocal、SWAP+SWAP ✅
6. **NeoVM identity ops** — x+0、x\*1、x|0、x^0 ✅
7. **NeoVM bool optimize** — x==true→x、x!=false→x ✅
8. **IR 常量折叠** — IR 层的常量传播 ✅

---

## 8. 结论

### 优点

1. **架构正确**: 语义等价转换而非逐 opcode 翻译，这是唯一正确的方式
2. **EVM→Neo 映射专业**: address.balance→GAS.balanceOf, address.code.length→isContract, address.transfer→GAS.transfer 等映射都很巧妙
3. **错误处理完善**: 丰富的 ErrorCode 体系、FixSuggestion、DiagnosticBuilder
4. **ERC→NEP 迁移指导**: 14+ 种模式检测，帮助开发者从 EVM 迁移
5. **原生合约覆盖完整**: 11 个原生合约的方法都有正确的 CallFlags 和返回值处理
6. **CALLT 优化**: 对原生合约调用使用更高效的 CALLT 指令
7. **低级调用支持**: address.call/staticcall 通过 TRY/ENDTRY 实现 (bool, bytes) 返回语义

### 需要关注

1. delegatecall 语义差异需要更强的 warning
2. msg.value 在非 onNEP17Payment 上下文中的行为需要明确
3. 内联汇编的静默 no-op 可能导致开发者误解
4. 函数重载名称修饰的边界情况

### 总体评级: **生产就绪 (Production Ready)**

编译器的核心编译路径正确，类型系统映射合理，NeoVM opcode 使用正确。发现的问题都是边界情况和文档层面的，没有会导致资金损失的严重 bug。
