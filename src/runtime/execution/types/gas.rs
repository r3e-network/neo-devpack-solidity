//! Gas tracking types for execution context.
//!
//! Provides structures for tracking gas consumption during execution.

/// Gas tracker for execution costs.
///
/// Per-opcode gas is charged via `spec::opcode_gas`; named-operation fallback
/// costs live in a static `match` (see `execution_gas::named_operation_cost`),
/// so the tracker itself is three plain counters — no per-instance
/// allocations.
#[derive(Debug)]
pub struct GasTracker {
    pub(crate) limit: u64,
    pub(crate) used: u64,
    pub(crate) base_cost: u64,
}
