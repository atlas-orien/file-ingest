use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// External or deferred asset reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum AssetReference {
    Uri(String),
    DataUrl(String),
    Placeholder(String),
}

/// Image placeholder or reference. OCR and vision text can be added later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub id: String,
    pub name: Option<String>,
    pub reference: AssetReference,
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
            reference: AssetReference::Placeholder(id.clone()),
            id,
            width: None,
            height: None,
            format: None,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    pub image: ImageRef,
}

impl ImageContent {
    pub fn new(image: ImageRef) -> Self {
        Self { image }
    }
}
