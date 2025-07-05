use eosio_shipper_gf::shipper_types::ProducerSchedule;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockDocument {
    #[serde(rename = "@timestamp")]
    pub timestamp: String, // Формат: "2022-12-31T12:34:56.789Z"
    pub block_num: u32,
    pub block_id: String,
    pub producer: String,
    pub new_producers: Option<ProducerSchedule>,
    pub schedule_version: u32,
    pub cpu_usage: u32,
    pub net_usage: u32,
}
// LOOK index_action.rs
//#[derive(Debug, Serialize, Deserialize)]
//pub struct ActionTrace {
//}