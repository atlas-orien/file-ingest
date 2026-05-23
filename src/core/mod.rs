//! Canonical document schema used by parsers, renderers, and chunkers.
//!
//! The core model keeps source order and provenance. Format-specific parsers
//! should emit this structure before any Markdown or AI-oriented rendering.

mod asset;
mod block;
mod document;
mod source;
mod table;

pub use asset::*;
pub use block::*;
pub use document::*;
pub use source::*;
pub use table::*;
