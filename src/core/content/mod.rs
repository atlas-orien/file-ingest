mod chart;
mod empty;
mod group;
mod image;
mod table;
mod text;

use serde::{Deserialize, Serialize};

pub use chart::*;
pub use empty::*;
pub use group::*;
pub use image::*;
pub use table::*;
pub use text::*;

/// Payload for a block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockContent {
    Text(TextContent),
    Table(TableContent),
    Chart(Chart),
    Image(ImageContent),
    Group(GroupContent),
    Empty(EmptyContent),
}

impl BlockContent {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(TextContent::new(text))
    }

    pub fn table(table: crate::core::Table) -> Self {
        Self::Table(TableContent::new(table))
    }

    pub fn chart(chart: Chart) -> Self {
        Self::Chart(chart)
    }

    pub fn image(image: crate::core::ImageRef) -> Self {
        Self::Image(ImageContent::new(image))
    }

    pub fn group(children: Vec<crate::core::Block>) -> Self {
        Self::Group(GroupContent::new(children))
    }

    pub fn empty() -> Self {
        Self::Empty(EmptyContent::default())
    }
}
