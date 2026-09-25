use super::*;

/// Neo runtime for executing compiled NeoVM bytecode
#[derive(Debug)]
pub struct NeoRuntime {
    pub(crate) execution_context: execution::ExecutionContext,
    pub(crate) state_manager: state::StateManager,
    pub(crate) storage_manager: storage::StorageManager,
    pub(crate) vm_bridge: bridge::VMBridge,
    pub(crate) gas_tracker: execution::GasTracker,
    /// Task #19: tracks whether `_deploy(null, false)` has been invoked
    /// on this runtime instance. `call_method` runs the deploy prologue
    /// exactly once so state-variable initializers populate storage
    /// before any user method observes it (subsumes Task #47).
    pub(crate) deploy_triggered: bool,
}

/// Runtime execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub exception: Option<RuntimeException>,
    pub state_changes: Vec<StateChange>,
    pub logs: Vec<LogEntry>,
    pub stack_trace: Option<Vec<StackFrame>>,
    pub metadata: ExecutionMetadata,
}

/// Runtime exception information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeException {
    pub exception_type: ExceptionType,
    pub message: String,
    pub instruction_pointer: Option<u32>,
    pub stack_trace: Vec<StackFrame>,
}

/// Types of runtime exceptions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExceptionType {
    OutOfGas,
    StackOverflow,
    StackUnderflow,
    InvalidOpcode,
    InvalidJump,
    RevertExecution,
    Fault,
    Halt,
}

impl ExceptionType {
    /// Check if this is a recoverable exception
    #[inline]
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::RevertExecution | Self::Halt)
    }

    /// Get exception name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OutOfGas => "OutOfGas",
            Self::StackOverflow => "StackOverflow",
            Self::StackUnderflow => "StackUnderflow",
            Self::InvalidOpcode => "InvalidOpcode",
            Self::InvalidJump => "InvalidJump",
            Self::RevertExecution => "RevertExecution",
            Self::Fault => "Fault",
            Self::Halt => "Halt",
        }
    }
}

/// State change record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateChange {
    pub change_type: StateChangeType,
    pub account: String,
    pub key: Option<Vec<u8>>,
    pub old_value: Option<Vec<u8>>,
    pub new_value: Vec<u8>,
}

impl StateChange {
    /// Check if this is a storage change
    #[inline]
    pub fn is_storage(&self) -> bool {
        self.change_type == StateChangeType::StorageChange
    }

    /// Get key as hex string
    #[inline]
    pub fn key_hex(&self) -> Option<String> {
        self.key.as_ref().map(hex::encode)
    }
}

/// Types of state changes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StateChangeType {
    BalanceChange,
    StorageChange,
    CodeChange,
    NonceChange,
    AccountCreation,
    AccountDeletion,
}

impl StateChangeType {
    /// Check if this is a destructive change
    #[inline]
    pub fn is_destructive(&self) -> bool {
        matches!(self, Self::AccountDeletion | Self::CodeChange)
    }

    /// Get change type as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BalanceChange => "balance",
            Self::StorageChange => "storage",
            Self::CodeChange => "code",
            Self::NonceChange => "nonce",
            Self::AccountCreation => "creation",
            Self::AccountDeletion => "deletion",
        }
    }
}

/// Log entry for events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub address: String,
    pub topics: Vec<Vec<u8>>,
    pub data: Vec<u8>,
}

impl LogEntry {
    /// Get the first topic (event signature) as hex
    pub fn event_signature_hex(&self) -> Option<String> {
        self.topics.first().map(hex::encode)
    }

    /// Get data as hex string
    pub fn data_hex(&self) -> String {
        hex::encode(&self.data)
    }

    /// Get number of indexed topics
    pub fn topic_count(&self) -> usize {
        self.topics.len()
    }
}

/// Stack frame for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub function_name: Option<String>,
    pub instruction_pointer: u32,
    pub opcode: String,
    pub stack_items: Vec<types::StackItem>,
    pub local_variables: HashMap<String, types::StackItem>,
}

impl StackFrame {
    /// Get stack depth
    pub fn stack_depth(&self) -> usize {
        self.stack_items.len()
    }

    /// Check if this frame has a function name
    pub fn has_function(&self) -> bool {
        self.function_name.is_some()
    }
}

/// Optional metadata overrides for a single execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionOverrides {
    pub block_height: Option<u64>,
    pub timestamp: Option<u64>,
    pub caller_account: Option<String>,
    /// Task #113: Solidity `msg.value` injected at the start of the next
    /// execution. `None` defaults to 0 (Neo N3 has no native "attached
    /// value"; this slot is purely a host-side override for Solidity
    /// source that reads `msg.value` inside a `payable` fallback, NEP-17
    /// `onPayment` hook, etc.). When set, the compiled `MsgValue`
    /// lowering syscall (`System.Runtime.GetMsgValue`) pushes this value.
    pub value: Option<u64>,
}

impl ExecutionOverrides {
    /// Check if any overrides are set
    pub fn has_overrides(&self) -> bool {
        self.block_height.is_some()
            || self.timestamp.is_some()
            || self.caller_account.is_some()
            || self.value.is_some()
    }

    /// Set block height
    pub fn with_block_height(mut self, height: u64) -> Self {
        self.block_height = Some(height);
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// Task #113: set `msg.value` for the next execution.
    pub fn with_value(mut self, value: u64) -> Self {
        self.value = Some(value);
        self
    }
}

/// Metadata captured from the execution environment
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionMetadata {
    pub block_height: Option<u64>,
    pub timestamp: Option<u64>,
    pub caller_account: Option<String>,
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub gas_limit: u64,
    pub call_stack_limit: u32,
    pub memory_limit: usize,
    pub storage_limit: usize,
    pub network_magic: u32,
    pub enable_debugging: bool,
    pub enable_tracing: bool,
    pub strict_mode: bool,
    pub neo_version: String,
    pub contract_account: String,
    pub default_block_height: u64,
    pub default_timestamp: u64,
}

/// Runtime errors
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Execution failed: {message}")]
    ExecutionError { message: String },

    #[error("Out of gas: used {used}, limit {limit}")]
    OutOfGas { used: u64, limit: u64 },

    /// An UNCATCHABLE VM fault — a NeoVM `ExecutionEngineLimits` violation
    /// (e.g. MaxTryNestingDepth) that corresponds to `VMState.FAULT` on a real
    /// node and must terminate execution immediately, NOT be routed into an
    /// enclosing `TRY`/catch. (A catchable limit fault lets bytecode build a
    /// fault-storm — found by the runtime_exec fuzzer.)
    #[error("VM fault: {message}")]
    VmFault { message: String },

    #[error("Stack overflow at depth {depth}")]
    StackOverflow { depth: u32 },

    #[error("Invalid operation: {operation}")]
    InvalidOperation { operation: String },

    #[error("State error: {message}")]
    StateError { message: String },

    #[error("Storage error: {message}")]
    StorageError { message: String },

    #[error("Bridge error: {message}")]
    BridgeError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

impl RuntimeError {
    /// Create an execution error
    pub fn execution(msg: impl Into<String>) -> Self {
        Self::ExecutionError {
            message: msg.into(),
        }
    }

    /// Create a storage error
    pub fn storage(msg: impl Into<String>) -> Self {
        Self::StorageError {
            message: msg.into(),
        }
    }

    /// Check if this is a gas-related error
    pub fn is_gas_error(&self) -> bool {
        matches!(self, Self::OutOfGas { .. })
    }
}

/// Runtime performance statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeStatistics {
    pub total_gas_used: u64,
    pub total_instructions_executed: u64,
    pub max_stack_depth: u32,
    pub storage_reads: u64,
    pub storage_writes: u64,
    pub state_changes: u64,
}

impl RuntimeStatistics {
    /// Get total storage operations
    pub fn total_storage_ops(&self) -> u64 {
        self.storage_reads + self.storage_writes
    }

    /// Get average gas per instruction
    pub fn avg_gas_per_instruction(&self) -> f64 {
        if self.total_instructions_executed == 0 {
            return 0.0;
        }
        self.total_gas_used as f64 / self.total_instructions_executed as f64
    }

    /// Merge with another statistics instance
    pub fn merge(&mut self, other: &RuntimeStatistics) {
        self.total_gas_used += other.total_gas_used;
        self.total_instructions_executed += other.total_instructions_executed;
        self.max_stack_depth = self.max_stack_depth.max(other.max_stack_depth);
        self.storage_reads += other.storage_reads;
        self.storage_writes += other.storage_writes;
        self.state_changes += other.state_changes;
    }
}
