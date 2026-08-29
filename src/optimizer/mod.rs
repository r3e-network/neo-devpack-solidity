//! IR Optimization Module
//!
//! Performs optimization passes on the intermediate representation (IR) before
//! bytecode generation. These optimizations reduce code size and improve runtime
//! performance.
//!
//! # Optimization Levels
//!
//! - **Level 0**: No IR optimization
//! - **Level 1**: Basic dead code elimination
//! - **Level 2**: Constant folding and propagation
//! - **Level 3**: NeoVM-specific optimizations (identity ops, boolean simplification)

use crate::ir;
use num_traits::{ToPrimitive, Zero};

mod optimize;
pub(crate) use optimize::*;
mod constant_folding;
pub(crate) use constant_folding::*;
mod labels;
pub(crate) use labels::*;
mod neovm;
pub(crate) use neovm::*;
