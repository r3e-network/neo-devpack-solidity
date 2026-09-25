//! Type definitions for execution context.
//!
//! This module organizes all the types used by the execution context into
//! logical submodules:
//!
//! - **stack**: StackItem enum and related serialization implementations
//! - **frame**: CallFrame and TryFrame for function calls and exception handling
//! - **state**: IteratorState, ContractState, and storage-related types
//! - **debug**: StepResult and MemoryChange for debugging support
//! - **gas**: GasTracker for gas consumption tracking
//! - **context**: ExecutionContext, the main execution context type

pub mod context;
pub mod debug;
pub mod frame;
pub mod gas;
pub mod stack;
pub mod state;

// Re-export commonly used types
pub use context::ExecutionContext;
pub use debug::{MemoryChange, StepResult};
pub use frame::{CallFrame, TryFrame, TryFrameState};
pub use gas::GasTracker;
pub use stack::StackItem;
pub use state::{
    ContractState, IsolatedStorageKey, IteratorState, LedgerBlock, LedgerTransaction,
    NotaryDeposit, OracleRequest, OverlayEntry, StorageOverlayEntries, StreamingCursor,
    TransactionSigner, WhitelistedFeeContract,
};
