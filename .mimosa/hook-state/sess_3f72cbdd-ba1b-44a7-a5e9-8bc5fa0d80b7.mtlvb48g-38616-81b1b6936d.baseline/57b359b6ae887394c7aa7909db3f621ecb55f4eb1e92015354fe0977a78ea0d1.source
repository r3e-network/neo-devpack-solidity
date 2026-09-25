//! Execution Context Module
//!
//! Provides execution context and gas tracking for Neo runtime.

use super::{spec, storage, LogEntry, RuntimeConfig, RuntimeError};
use ripemd::Ripemd160;
use serde_json;
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use std::collections::{HashMap, HashSet};

use std::ptr::NonNull;

pub mod types;

mod collections;
mod execution_gas;
mod execution_impl_part1_debug;
mod execution_impl_part1_init;
mod execution_impl_part1_native;
mod execution_impl_part1_slots;
mod execution_impl_part1_stack_api;
mod execution_impl_part1_stack_ops;
mod execution_impl_part1_state;
mod execution_impl_part2_contract_call;
mod execution_impl_part2_native;
mod execution_impl_part3_bytes;
mod execution_impl_part3_conversion;
mod execution_impl_part3_logical;
mod execution_impl_part3_offsets;
mod execution_stack_item;
mod helpers;
mod instruction;
mod syscalls;

// Re-export all types for backward compatibility
pub use types::*;
