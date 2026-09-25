use super::*;

/// VM Bridge for EVM-to-NeoVM translation
#[derive(Debug)]
pub struct VMBridge {
    pub(crate) config: RuntimeConfig,
    pub(crate) contract_account: String,
}

/// VM Bridge errors
#[derive(Debug, Error)]
pub enum VMBridgeError {
    #[error("Instruction not supported: {opcode:#04x}")]
    UnsupportedInstruction { opcode: u8 },

    #[error("System call failed: {name} - {message}")]
    SystemCallFailed { name: String, message: String },

    #[error("Stack operation failed: {message}")]
    StackOperationFailed { message: String },

    #[error("Memory operation failed: {message}")]
    MemoryOperationFailed { message: String },

    #[error("Storage operation failed: {message}")]
    StorageOperationFailed { message: String },

    #[error("State operation failed: {message}")]
    StateOperationFailed { message: String },

    #[error("Bridge error: {message}")]
    BridgeError { message: String },

    #[error("Invalid argument count: expected {expected}, got {got}")]
    InvalidArguments { expected: usize, got: usize },
}
