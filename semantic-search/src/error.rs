//! # Error module
//!
//! Possible errors.

use reqwest::Error as ReqwestError;
use std::array::TryFromSliceError;

/// Possible errors.
#[derive(Debug)]
pub enum SenseError {
    /// Embedding must be 1024-dimensional.
    DimensionMismatch,
    /// Malformed API key.
    MalformedApiKey,
    /// Request failed.
    RequestFailed(ReqwestError),
}

impl std::fmt::Display for SenseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionMismatch => write!(f, "Embedding must be 1024-dimensional"),
            Self::MalformedApiKey => write!(f, "Malformed API key"),
            Self::RequestFailed(error) => write!(f, "Request failed: {}", error),
        }
    }
}

impl std::error::Error for SenseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RequestFailed(error) => Some(error),
            _ => None,
        }
    }
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
