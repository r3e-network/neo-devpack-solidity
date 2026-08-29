//! Storage Management Module
//!
//! Provides persistent storage for smart contracts with Neo blockchain compatibility.

use super::{RuntimeConfig, RuntimeError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

mod storage_impl;
mod storage_types;

pub use storage_types::*;

#[cfg(test)]
mod tests;
