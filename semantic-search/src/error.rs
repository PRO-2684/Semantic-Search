//! # Error module
//!
//! Possible errors.

use base64::DecodeError;
use doc_for::doc_impl;
use reqwest::{Error as ReqwestError, header::InvalidHeaderValue};
use std::array::TryFromSliceError;
use thiserror::Error;

/// Possible errors.
#[doc_impl(strip = 1, doc_for = false, gen_attr = "error({doc})")]
#[derive(Debug, Error)]
pub enum SenseError {
    /// Embedding must be 1024-dimensional.
    DimensionMismatch,
    /// Malformed API key.
    MalformedApiKey,
    /// Request failed.
    RequestFailed {
        /// Source of the error.
        source: ReqwestError,
    },
    /// Invalid header value.
    InvalidHeaderValue,
    /// Base64 decoding failed.
    Base64DecodingFailed,
}

impl From<ReqwestError> for SenseError {
    /// Error when request fails.
    fn from(error: ReqwestError) -> Self {
        Self::RequestFailed { source: error }
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

impl From<InvalidHeaderValue> for SenseError {
    /// Error when header value is invalid.
    fn from(_: InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue
    }
}

impl From<DecodeError> for SenseError {
    /// Error when base64 decoding fails.
    fn from(_: DecodeError) -> Self {
        Self::Base64DecodingFailed
    }
}
