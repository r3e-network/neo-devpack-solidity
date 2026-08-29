use super::*;

/// IR module representing a compiled contract
#[derive(Debug)]
pub struct Module {
    pub functions: Vec<Function>,
    pub state_variables: Vec<StateVariable>,
    pub events: Vec<Event>,
}

impl Module {
    /// Get the constructor function if present
    pub fn constructor(&self) -> Option<&Function> {
        self.functions
            .iter()
            .find(|f| f.kind == FunctionKind::Constructor)
    }

    /// Get a function by name
    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Count total instructions across all functions
    pub fn instruction_count(&self) -> usize {
        self.functions
            .iter()
            .flat_map(|f| &f.basic_blocks)
            .map(|bb| bb.instructions.len())
            .sum()
    }
}

/// IR function representation
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub kind: FunctionKind,
    pub parameters: Vec<ValueType>,
    pub returns: Vec<ValueType>,
    pub basic_blocks: Vec<BasicBlock>,
    pub local_count: u16,
}

impl Function {
    /// Check if this is a constructor
    pub fn is_constructor(&self) -> bool {
        self.kind == FunctionKind::Constructor
    }

    /// Get total instruction count
    pub fn instruction_count(&self) -> usize {
        self.basic_blocks
            .iter()
            .map(|bb| bb.instructions.len())
            .sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind {
    Constructor,
    Regular,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateVariable {
    pub name: Option<String>,
    pub ty: ValueType,
    pub is_constant: bool,
    pub is_immutable: bool,
    pub storage_key: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub name: String,
}

/// Per-parameter metadata for an EVM-canonical event signature.
///
/// Used by the IR lowering to compute the `keccak256(canonical_signature)`
/// topic hash and to distinguish indexed-static args (emitted verbatim as
/// 32-byte big-endian slots) from indexed-dynamic args (`string`/`bytes`,
/// emitted as `keccak256(value)`), and from non-indexed args (routed through
/// `abi.encode` into the log's `data` payload).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventParamInfo {
    /// Canonical Solidity type string for this parameter, with aliases
    /// normalized (e.g., `uint` → `uint256`, `bytes1` → `bytes1`).
    pub canonical_type: String,
    /// Whether the parameter is declared `indexed` in Solidity source.
    pub indexed: bool,
    /// Whether the canonical type is a dynamic type for topic hashing
    /// purposes. `string` / `bytes` / dynamic arrays all hash their value
    /// when indexed; everything else is left-padded to 32 bytes verbatim.
    pub is_dynamic: bool,
}

/// Precomputed EVM-canonical event signature metadata.
///
/// Built once per event at module lowering time so `emit` sites can push
/// `topic[0]` as a 32-byte literal (no runtime hashing) and branch on each
/// parameter's `indexed` / `is_dynamic` flags without re-deriving them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSignature {
    /// Canonical signature string `Name(type1,type2,...)` — matches Solidity
    /// `bytes4 selector = keccak256(...)` convention and is fed to keccak to
    /// produce `topic[0]`.
    pub canonical: String,
    /// `keccak256(canonical)` — 32 bytes, BE-layout, pushed as `topic[0]`.
    /// For anonymous events this field is still populated (for completeness)
    /// but the lowering skips the topic0 prepend per the EVM ABI.
    pub topic0: [u8; 32],
    /// Per-parameter metadata in declaration order.
    pub params: Vec<EventParamInfo>,
    /// Whether the event was declared `anonymous` in Solidity source. Per
    /// the EVM ABI, anonymous events suppress the `keccak256(signature)`
    /// topic0 slot so they can carry up to 4 indexed topics (vs. 3 for
    /// non-anonymous).
    pub is_anonymous: bool,
}

/// Declared custom `error` signature, keyed by error name in the lowering
/// context's `error_signature_map`.
///
/// Revert-site lowering uses the DECLARED parameter types (not the types
/// inferred from argument expressions) to compute the EVM-canonical 4-byte
/// selector `keccak256("Name(t1,t2,...)")[..4]`, and the declared parameter
/// names to reorder `revert E({b: .., a: ..})` named arguments into
/// declaration order before encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorAbiSignature {
    /// Declared parameter names in declaration order (`None` if unnamed).
    pub param_names: Vec<Option<String>>,
    /// Declared parameter Solidity type strings in declaration order (raw
    /// source form; canonicalized against enums/structs in scope at
    /// lowering time).
    pub param_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeValue {
    MsgSender,
    MsgValue,
    MsgData,
    TxOrigin,
    BlockTimestamp,
    BlockNumber,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Drop(ValueType),
    /// NeoVM DUP - duplicate the top stack item
    Dup,
    /// NeoVM SWAP - swap the top two stack items
    Swap,
    LoadParameter(usize),
    // Task #156 — write-to-parameter path. Solidity permits assigning to function
    // parameters (`a = 5;` where `a` is a param); NeoVM exposes STARG/STARG0..6
    // for this. Emitted by `lower_assignment` for single-variable and tuple-LVALUE
    // assignments whose target is a named parameter (so `(a, b) = (b, a)` swaps in
    // place without clobbering the original binding).
    StoreParameter(usize),
    PushLiteral(LiteralValue),
    Return,
    ReturnVoid,
    ReturnDefault(ValueType),
    BinaryOp(BinaryOperator),
    LoadState(usize),
    StoreState(usize),
    LoadStorageDynamic,
    LoadLocal(usize),
    StoreLocal(usize),
    LoadMappingElement {
        state_index: usize,
        key_types: Vec<ValueType>,
    },
    StoreMappingElement {
        state_index: usize,
        key_types: Vec<ValueType>,
    },
    /// Task #205 — store a dynamic-array value into storage using the same
    /// layout the read side expects: length at the base slot and elements at
    /// `keccak256(serialize(i) || slot)`.
    StoreArrayDeepCopy {
        state_index: usize,
        key_types: Vec<ValueType>,
    },
    LoadStructField {
        state_index: usize,
        key_types: Vec<ValueType>,
        field_keys: Vec<[u8; 32]>,
        field_type: ValueType,
    },
    StoreStructField {
        state_index: usize,
        key_types: Vec<ValueType>,
        field_keys: Vec<[u8; 32]>,
        field_type: ValueType,
    },
    LoadStructArrayElement {
        state_index: usize,
        key_types: Vec<ValueType>,
        field_keys: Vec<[u8; 32]>,
        element_type: ValueType,
    },
    StoreStructArrayElement {
        state_index: usize,
        key_types: Vec<ValueType>,
        field_keys: Vec<[u8; 32]>,
        element_type: ValueType,
    },
    // Task #82: nested mapping inside struct-field access, e.g. `slots[k].balances[a]`.
    // Derives a storage slot by composing the outer mapping keys + struct field path +
    // inner mapping key chain, then performs a scalar Get/Put.
    LoadStructFieldMappingElement {
        state_index: usize,
        key_types: Vec<ValueType>,
        field_keys: Vec<[u8; 32]>,
        trailing_key_types: Vec<ValueType>,
        value_type: ValueType,
    },
    StoreStructFieldMappingElement {
        state_index: usize,
        key_types: Vec<ValueType>,
        field_keys: Vec<[u8; 32]>,
        trailing_key_types: Vec<ValueType>,
    },
    LoadRuntimeValue(RuntimeValue),
    GetSize,
    CallFunction {
        name: String,
        arg_count: usize,
    },
    /// Task #186 — push an internal function's bytecode offset as an integer
    /// literal. Emitted when a function name is evaluated as a value (e.g.
    /// passed as an argument of function-pointer type). The bytecode emitter
    /// fixes up the placeholder with the function's actual `method.offset`
    /// once all methods have been laid out.
    PushFunctionOffset {
        name: String,
    },
    /// Task #186 — indirect call through a function pointer (internal
    /// `function(...) internal returns (...)` passed as a parameter). Pops
    /// `arg_count` argument values and then the function-offset value, reverses
    /// the argument order so the callee's `INITSLOT` binds them left-to-right,
    /// then emits `CALLA` (0x36) which jumps to the target offset.
    CallIndirect {
        arg_count: usize,
        has_return: bool,
    },
    CallBuiltin {
        builtin: BuiltinCall,
        arg_count: usize,
    },
    EmitEvent {
        event_index: usize,
        arg_count: usize,
    },
    EmitEventByName {
        name: String,
        arg_count: usize,
    },
    /// NeoVM `CONVERT` opcode. Converts the top stack item to a different StackItemType.
    Convert {
        target: ConvertTarget,
    },
    /// NeoVM `ISTYPE` opcode. Checks whether the top stack item matches a StackItemType and pushes a boolean.
    IsType {
        target: ConvertTarget,
    },
    /// Allocate a new mutable buffer (NeoVM `NEWBUFFER`) whose length is taken from the stack.
    /// Used to implement Solidity `new bytes(n)` / `new string(n)` allocations.
    NewBuffer,
    NewArray {
        element_type: ValueType,
    },
    /// Allocate a new empty map (NeoVM `NEWMAP`). Used by Task #100 yul
    /// `tstore`/`tload` lowering to back `__yul_transient` with an in-memory
    /// key-value store that lives for the duration of the current invocation.
    NewMap,
    ArrayGet,
    ArraySet,
    /// NeoVM `HASKEY` opcode: pops `[collection, key]`, pushes `Boolean` — `true`
    /// if the key exists in the collection (Array index in range or Map key
    /// present). Used by Task #100 `tload` to return 0 for unset slots instead
    /// of raising `PICKITEM: key not found`.
    HasKey,
    /// NeoVM MEMCPY opcode: copy a slice of bytes into a buffer.
    /// Stack order (bottom -> top): [dst, dst_offset, src, src_offset, count]
    MemCpy,
    /// NeoVM SUBSTR opcode: pop `count` and `index`, then a ByteString,
    /// push the substring `bytes[index .. index + count]` as a ByteString.
    /// Stack order (bottom -> top): `[bytes, index, count]` -> `[bytes_substr]`.
    /// Used to implement contiguous `bytes memory`/`bytes calldata` slicing
    /// so `b[a:b]` returns a raw ByteString rather than an Array of bytes.
    Substr,
    /// NeoVM REVERSEITEMS opcode: reverse an Array/Buffer in place.
    /// Note: consumes one reference and does not push it back.
    ReverseItems,
    BitwiseNot,
    /// NeoVM NOT (0xAA) — logical boolean negation.
    /// Pops one value, pushes `!value.is_truthy()`.
    /// Correctly handles null (returns true), unlike `PUSHF + EQUAL`.
    LogicalNot,
    Try {
        catch_target: usize,
    },
    EndTry {
        target: usize,
    },
    Jump {
        target: usize,
    },
    JumpIf {
        target: usize,
    },
    Label(usize),
    /// NeoVM THROW - raises a catchable exception using the value on the stack.
    Throw,
    AbortMsg,
    Abort,
}

/// Literal values in IR
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralValue {
    Integer(BigInt),
    Boolean(bool),
    String(Vec<u8>),
    ByteArray(Vec<u8>),
    Address(Vec<u8>),
    Null,
}

impl LiteralValue {
    /// Create an integer literal
    pub fn int(value: impl Into<BigInt>) -> Self {
        Self::Integer(value.into())
    }

    /// Create a boolean literal
    pub fn bool(value: bool) -> Self {
        Self::Boolean(value)
    }

    /// Check if this is a null value
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Get the type name
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Integer(_) => "integer",
            Self::Boolean(_) => "boolean",
            Self::String(_) => "string",
            Self::ByteArray(_) => "bytes",
            Self::Address(_) => "address",
            Self::Null => "null",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvertTarget {
    Any,
    Boolean,
    Integer,
    ByteArray,
    Array,
    Map,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    Integer {
        signed: bool,
        bits: u16,
    },
    Boolean,
    String,
    Address,
    ByteArray {
        fixed_len: Option<u16>,
    },
    Array(Box<ValueType>),
    Mapping {
        key: Box<ValueType>,
        value: Box<ValueType>,
    },
    Struct {
        name: String,
        fields: Vec<StructField>,
    },
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestType {
    Integer,
    Boolean,
    String,
    Hash160,
    Hash256,
    ByteArray,
    Array,
    Map,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub ty: ValueType,
    pub key: [u8; 32],
}

/// Binary operators for IR instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl BinaryOperator {
    /// Check if this is an arithmetic operator
    pub fn is_arithmetic(&self) -> bool {
        matches!(
            self,
            Self::Add | Self::Sub | Self::Mul | Self::Div | Self::Mod
        )
    }

    /// Check if this is a comparison operator
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            Self::Lt | Self::Le | Self::Gt | Self::Ge | Self::Eq | Self::Ne
        )
    }

    /// Check if this is a bitwise operator
    pub fn is_bitwise(&self) -> bool {
        matches!(
            self,
            Self::BitAnd | Self::BitOr | Self::BitXor | Self::Shl | Self::Shr
        )
    }

    /// Get the operator symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::Shl => "<<",
            Self::Shr => ">>",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Eq => "==",
            Self::Ne => "!=",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeContract {
    Neo,
    Gas,
    ContractManagement,
    Policy,
    Oracle,
    RoleManagement,
    Notary,
    Treasury,
    Ledger,
    CryptoLib,
    StdLib,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinCall {
    RuntimeNotify,
    RuntimeCheckWitness,
    AbiEncode,
    AbiEncodePacked,
    AbiEncodeCall,
    AbiEncodeWithSignature,
    AbiDecode,
    Keccak256,
    Ecrecover,
    StorageFind,
    StoragePut,
    StorageGet,
    StorageDelete,
    ContractCall,
    ContractCallWithFlags,
    NotifySerialized,
    VerifySignature,
    /// Convenience wrapper for `ContractManagement.deploy` that returns only the deployed
    /// contract hash (UInt160) rather than the full `ContractState`.
    DeployContract,
    /// Convenience wrapper for `ContractManagement.getContract` that reshapes the returned
    /// `ContractState` into a devpack-friendly struct:
    /// `[hash, nef, serialize(manifest), updateCounter]`.
    GetContract,
    /// Convenience wrapper for fetching the NEF file from a contract state.
    /// This is exposed as `Syscalls.getContractScript` in the devpack.
    GetContractScript,
    /// Convenience wrapper for `NeoToken.getAccountState` that:
    /// - Returns a default struct when the native contract returns `null`.
    /// - Normalizes the `voteTo` field to an empty byte string when it is `null`.
    GetNeoAccountState,
    NativeCall {
        contract: NativeContract,
        method: String,
    },
    Syscall(String),
    TypeOf,
    /// `bytes.concat(a, b, ...)` / `string.concat(a, b, ...)` — chains NeoVM CAT opcodes.
    BytesConcat,
    /// Task #H6a: route `address(0x01).staticcall(abiPayload)` to the
    /// `recoverSecp256K1 → keccak256 → RIGHT 20` sequence (same lowering as
    /// Solidity `ecrecover`), then left-pad the 20-byte result to 32 bytes so
    /// the returned `bytes memory` matches the 0x01 EVM precompile contract.
    /// Takes one argument: the 128-byte `bytes memory` payload
    /// `abi.encode(hash, v, r, s)`.
    PrecompileEcrecover,
    /// Task #H6b: route `address(0x05).staticcall(abiPayload)` to the NeoVM
    /// MODPOW opcode. Takes one argument: an ABI payload laid out as
    /// `base_len || exp_len || mod_len || base || exp || mod` (Ethereum 0x05
    /// modexp contract). Produces a `mod_len`-wide big-endian byte string
    /// left-padded into a 32-byte slot when `mod_len <= 32`.
    PrecompileModexp,
}
