use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AbiDocument {
    #[serde(rename = "@timestamp")]
    pub timestamp: String, // Формат: "2022-12-31T12:34:56.789Z"
    pub account: String,
    pub block: u32,
    pub abi: String,      // JSON-строка
    pub abi_hex: String,
    pub actions: Vec<String>,
    pub tables: Vec<String>,
}