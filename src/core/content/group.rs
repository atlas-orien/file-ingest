use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::core::Block;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupContent {
    pub children: Vec<Block>,
    pub metadata: HashMap<String, Value>,
}

impl GroupContent {
    pub fn new(children: Vec<Block>) -> Self {
        Self {
            children,
            metadata: HashMap::new(),
        }
    }
}
