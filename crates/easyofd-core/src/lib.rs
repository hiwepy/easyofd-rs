//! # easyofd-core
//!
//! Core types, traits, and models for the easyofd-rs OFD document library.
//!
//! This crate provides:
//! - [`model`] — OFD data model types (`OfdPage`, `TextObject`, `ImageObject`, etc.)
//! - [`ofd_model`] — The [`OfdModel`] trait for mapping Rust types to OFD pages
//! - [`error`] — Error types ([`OfdError`])
//! - [`page_size`] — Common page size constants

pub mod error;
pub mod model;
pub mod ofd_model;

// Re-export core types at crate root for convenience.
pub use error::{OfdError, OfdResult};
pub use model::{
    ContentObject, ImageFormat, ImageObject, OfdMetadata, OfdPage, PathObject, TextObject,
    Watermark, page_size,
};
pub use ofd_model::{OfdField, OfdFieldKind, OfdModel};
