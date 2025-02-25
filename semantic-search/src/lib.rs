//! # Semantic Search Library
//!
//! `semantic-search` is a library for searching semantically similar documents.
//!
//! To be specific, it helps you get embeddings of texts and search for top-k similar texts, where similarity is defined by cosine similarity of embeddings.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

pub mod embedding;
mod error;
mod silicon_flow;

pub use embedding::Embedding;
pub use error::SenseError;
