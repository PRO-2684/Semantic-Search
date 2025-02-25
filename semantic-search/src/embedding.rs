//! Embedding representation, conversion and calculation. Assumes little-endian byte order.
//!
//! # Representation
//!
//! Embedding is represented as a 1024-dimensional vector of 32-bit floating point numbers. [`Embedding`] struct is a wrapper around  [`EmbeddingRaw`] (alias for `[f32; 1024]`), and provides methods for conversion and calculation.
//!
//! # Conversion
//!
//! - [`EmbeddingBytes`] is a 1024 * 4 bytes array.
//! - [`Embedding`] can be converted to [`EmbeddingBytes`] and vice versa.
//! - `Vec<f32>` and `Vec<u8>` can be converted to [`Embedding`], but [`DimensionMismatch`](SenseError::DimensionMismatch) error is returned if the length mismatches.
//!
//! # Calculation
//!
//! Cosine similarity between two embeddings can be calculated using [`cosine_similarity`](Embedding::cosine_similarity) method.

use super::SenseError;
use std::{
    convert::TryFrom,
    ops::{Deref, DerefMut},
};

/// Raw embedding representation.
pub type EmbeddingRaw = [f32; 1024];

/// Embedding represented in bytes (little-endian).
pub type EmbeddingBytes = [u8; 1024 * 4];

/// Embedding representation.
pub struct Embedding(EmbeddingRaw);

// Cosine similarity calculation

impl Embedding {
    /// Calculate cosine similarity between two embeddings.
    #[must_use]
    pub fn cosine_similarity(&self, other: &Self) -> f32 {
        let dot_product = self
            .iter()
            .zip(other.iter())
            .map(|(a, b)| a * b)
            .sum::<f32>();
        let norm_self = self.iter().map(|a| a * a).sum::<f32>().sqrt();
        let norm_other = other.iter().map(|b| b * b).sum::<f32>().sqrt();
        dot_product / (norm_self * norm_other)
    }
}

// Convertion

impl From<EmbeddingBytes> for Embedding {
    /// Convert 1024 * 4 bytes to `Embedding` (little-endian).
    fn from(bytes: EmbeddingBytes) -> Self {
        let mut embedding = [0.0; 1024];
        bytes.chunks_exact(4).enumerate().for_each(|(i, chunk)| {
            let f = f32::from_le_bytes(chunk.try_into().unwrap()); // Safe to unwrap, as we know the length is 4
            embedding[i] = f;
        });
        Self(embedding)
    }
}

impl From<Embedding> for EmbeddingBytes {
    /// Convert `Embedding` to 1024 * 4 bytes (little-endian).
    fn from(embedding: Embedding) -> Self {
        let mut bytes = [0; 1024 * 4];
        bytes
            .chunks_exact_mut(4)
            .enumerate()
            .for_each(|(i, chunk)| {
                let f = embedding.0[i];
                chunk.copy_from_slice(&f.to_le_bytes());
            });
        bytes
    }
}

impl TryFrom<Vec<f32>> for Embedding {
    type Error = SenseError;

    /// Convert `Vec<f32>` to `Embedding`.
    ///
    /// # Errors
    ///
    /// Returns [`DimensionMismatch`](SenseError::DimensionMismatch) if the length of the input vector is not 1024.
    fn try_from(value: Vec<f32>) -> Result<Self, Self::Error> {
        let embedding = value
            .try_into()
            .map_err(|_| SenseError::DimensionMismatch)?;
        Ok(Self(embedding))
    }
}

impl TryFrom<Vec<u8>> for Embedding {
    type Error = SenseError;

    /// Convert `Vec<u8>` to `Embedding`.
    ///
    /// # Errors
    ///
    /// Returns [`DimensionMismatch`](SenseError::DimensionMismatch) if the length of the input vector is not 1024 * 4.
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let bytes: EmbeddingBytes = value
            .try_into()
            .map_err(|_| SenseError::DimensionMismatch)?;
        Ok(Self::from(bytes))
    }
}

// Implement `Deref` and `DerefMut` for `Embedding`

impl Deref for Embedding {
    type Target = EmbeddingRaw;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Embedding {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMBEDDING_FLOAT: f32 = 1.14; // 0x3F91EB85
    const EMBEDDING_CHUNK: [u8; 4] = [0x85, 0xEB, 0x91, 0x3F];

    #[test]
    fn embedding_from_bytes() {
        let mut bytes = [0; 1024 * 4];
        bytes.chunks_exact_mut(4).for_each(|chunk| {
            chunk.copy_from_slice(&EMBEDDING_CHUNK);
        });

        let embedding = Embedding::from(bytes);
        embedding
            .iter()
            .for_each(|&f| assert_eq!(f, EMBEDDING_FLOAT));
    }

    #[test]
    fn bytes_from_embedding() {
        let embedding = Embedding([EMBEDDING_FLOAT; 1024]);
        let bytes = EmbeddingBytes::from(embedding);

        bytes.chunks_exact(4).for_each(|chunk| {
            assert_eq!(chunk, EMBEDDING_CHUNK);
        });
    }

    #[test]
    fn similar_to_self() {
        let embedding = Embedding([EMBEDDING_FLOAT; 1024]);
        let similarity = embedding.cosine_similarity(&embedding);
        let delta = (similarity - 1.0).abs();
        // Approximate equality
        assert!(delta <= f32::EPSILON);
    }
}
