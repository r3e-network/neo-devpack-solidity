//! State Management Module
//!
//! Manages account states, balances, and transaction state for Neo runtime.

use super::{RuntimeConfig, RuntimeError, StateChange, StateChangeType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod state_impl;
mod state_types;

pub use state_types::*;

#[cfg(test)]
mod tests;
