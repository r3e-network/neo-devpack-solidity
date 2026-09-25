//! Diagnostic recording capability trait for IR lowering.
//!
//! This trait defines the interface for recording errors, warnings, and
//! suggestions during IR lowering. It is the smallest, most self-contained
//! capability extracted from `LoweringContext` — the first step in decomposing
//! the God struct into focused capability traits.
//!
//! Any type that implements `DiagnosticContext` can be used as the error-recording
//! sink for lowering functions that only need to report diagnostics, without
//! depending on the full `LoweringContext`.

/// Capability trait for recording compilation diagnostics during IR lowering.
///
/// The smallest capability extracted from `LoweringContext` — the first step
/// in decomposing that God struct into focused capability traits. A lowering
/// helper that only needs to report a fatal error can take
/// `&mut impl DiagnosticContext` instead of the full `LoweringContext`.
///
/// Only `record_error` is exposed today (the sole method a trait-generic
/// consumer needs). Warning / suggestion / `take_*` recording still lives as
/// inherent methods on `LoweringContext`; promote them onto this trait when a
/// consumer generic over `DiagnosticContext` actually needs them.
pub(crate) trait DiagnosticContext {
    /// Record a fatal error message.
    fn record_error(&mut self, message: impl Into<String>);
}
