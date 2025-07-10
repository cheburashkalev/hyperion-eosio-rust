use eosio_shipper_gf::shipper_types::{AccountAuthSequence, PermissionLevel, ProducerSchedule};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_id: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ActionReceipt {
    pub receiver: String,
    pub global_sequence: String,
    pub recv_sequence: String,
    pub auth_sequence: Vec<AccountAuthSequence>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ActionDocument {
    #[serde(rename = "@timestamp")]
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<String>>,
    pub act: HyperionActionAct,
    pub block_num: u32,
    pub block_id: String,
    pub global_sequence: String,
    pub producer: String,
    pub trx_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_ram_deltas: Option<Vec<Value>>,
    

    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_free: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub except: Option<String>,
    
    pub receipts: Vec<ActionReceipt>,
    pub creator_action_ordinal: u32,
    pub action_ordinal: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_usage_us: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_usage_words: Option<u32>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_count: Option<u32>,
    pub inline_filtered: bool,
    pub act_digest: String,
}
#[derive(Serialize, Deserialize, Debug,Default)]
pub struct HyperionActionAct {
    pub account: String,
    pub name: String,
    pub authorization: Vec<PermissionLevel>,
    pub data: Value
}