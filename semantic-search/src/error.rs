//! # Error module
//!
//! Possible errors.

/// Possible errors.
pub enum SenseError {
    /// Embedding must be 1024-dimensional.
    DimensionMismatch,
}
