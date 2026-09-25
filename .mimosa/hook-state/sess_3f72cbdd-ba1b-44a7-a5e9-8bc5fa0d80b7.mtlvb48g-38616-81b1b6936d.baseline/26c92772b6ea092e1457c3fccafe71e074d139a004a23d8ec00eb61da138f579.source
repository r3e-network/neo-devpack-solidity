//! VM Bridge Module
//!
//! Provides bridge between EVM semantics and NeoVM execution environment.

use super::{
    execution, state, storage, ExceptionType, ExecutionMetadata, ExecutionResult, RuntimeConfig,
    RuntimeError, RuntimeException, StackFrame, StateChange,
};
use std::collections::HashMap;
use thiserror::Error;

mod bridge_impl_core;
mod bridge_impl_stack_items;
mod bridge_types;

pub use bridge_types::*;
