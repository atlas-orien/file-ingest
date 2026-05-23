//! Canonical document schema used by parsers, renderers, and chunkers.
//!
//! The core model keeps source order and provenance. Format-specific parsers
//! should emit this structure before any Markdown or AI-oriented rendering.

mod block;
mod content;
mod document;
mod source;

pub use block::*;
pub use content::*;
pub use document::*;
pub use source::*;
