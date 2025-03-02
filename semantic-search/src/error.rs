//! # Error module
//!
//! Possible errors.

use reqwest::Error as ReqwestError;
use std::array::TryFromSliceError;
use thiserror::Error;

/// Possible errors.
#[derive(Debug, Error)]
pub enum SenseError {
    /// Embedding must be 1024-dimensional.
    #[error("Embedding must be 1024-dimensional")]
    DimensionMismatch,
    /// Malformed API key.
    #[error("Malformed API key")]
    MalformedApiKey,
    /// Request failed.
    #[error("Request failed: {0}. Make sure the API key is correct.")]
    RequestFailed(ReqwestError),
}

impl From<ReqwestError> for SenseError {
    /// Error when request fails.
    fn from(error: ReqwestError) -> Self {
        Self::RequestFailed(error)
    }
}

impl From<TryFromSliceError> for SenseError {
    /// Error when casting slice to array (length mismatch).
    fn from(_: TryFromSliceError) -> Self {
        Self::DimensionMismatch
    }
}

impl From<Vec<u8>> for SenseError {
    /// Error when casting `Vec<u8>` to array (length mismatch).
    fn from(_: Vec<u8>) -> Self {
        Self::DimensionMismatch
    }
}

impl From<Vec<f32>> for SenseError {
    /// Error when casting `Vec<f32>` to array (length mismatch).
    fn from(_: Vec<f32>) -> Self {
        Self::DimensionMismatch
    }
}
