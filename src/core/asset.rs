use crate::model::ImageReference;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Image placeholder or reference. OCR and vision text can be added later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub id: String,
    pub name: Option<String>,
    pub reference: ImageReference,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub metadata: HashMap<String, Value>,
}

impl ImageRef {
    pub fn placeholder(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: Some(id.clone()),
            reference: ImageReference::Placeholder(id.clone()),
            id,
            width: None,
            height: None,
            format: None,
            metadata: HashMap::new(),
        }
    }
}
