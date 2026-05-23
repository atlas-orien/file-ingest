use serde::{Deserialize, Serialize};

/// Original location of a block inside the source document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceLocation {
    pub page: Option<u32>,
    pub sheet: Option<String>,
    pub row_range: Option<RangeU32>,
    pub col_range: Option<RangeU32>,
    pub bbox: Option<BBox>,
    /// Format-specific path, such as `word/document.xml` or `xl/worksheets/sheet1.xml`.
    pub path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeU32 {
    pub start: u32,
    pub end: u32,
}

impl RangeU32 {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

/// Bounding box in source coordinate space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
