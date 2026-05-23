use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Structured data behind a chart. This stores chart data, not chart images.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Chart {
    pub title: Option<String>,
    pub chart_type: ChartType,
    pub series: Vec<ChartSeries>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Line,
    Bar,
    Column,
    Pie,
    Scatter,
    Area,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChartSeries {
    pub name: Option<String>,
    pub categories: Vec<String>,
    pub values: Vec<String>,
    pub category_range: Option<String>,
    pub value_range: Option<String>,
    pub metadata: HashMap<String, Value>,
}
