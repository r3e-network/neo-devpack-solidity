//! Neo N3 opcode/syscall/native-contract registry

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::interop::interop_id_bytes;

/// Opcode metadata (lightweight – only what the runtime needs today).
#[derive(Debug, Clone, Copy)]
pub struct OpcodeSpec {
    pub code: u8,
    pub name: &'static str,
    pub gas: u64,
}

impl OpcodeSpec {
    /// Check if this is a push opcode
    pub fn is_push(&self) -> bool {
        self.code <= 0x20 || (self.code >= 0x0C && self.code <= 0x0E)
    }

    /// Check if this is a jump opcode
    pub fn is_jump(&self) -> bool {
        self.code >= 0x22 && self.code <= 0x33
    }

    /// Check if this is a call opcode
    pub fn is_call(&self) -> bool {
        self.code >= 0x34 && self.code <= 0x37
    }
}

/// Syscall metadata.
#[derive(Debug, Clone, Copy)]
pub struct SyscallSpec {
    pub id: [u8; 4],
    pub name: &'static str,
}

impl SyscallSpec {
    /// Get the syscall ID as hex string
    pub fn id_hex(&self) -> String {
        hex::encode(self.id)
    }
}

/// Native contract metadata.
#[derive(Debug, Clone, Copy)]
pub struct NativeContractSpec {
    pub hash: [u8; 20],
    pub name: &'static str,
}

impl NativeContractSpec {
    /// Get the contract hash as hex string
    pub fn hash_hex(&self) -> String {
        hex::encode(self.hash)
    }
}

mod gas;
mod native_contracts;
mod opcodes;
mod syscalls;

pub use gas::*;
pub use native_contracts::*;
pub use opcodes::*;
pub use syscalls::*;

#[cfg(test)]
mod tests;
