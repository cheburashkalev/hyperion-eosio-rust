use serde_json::{json, Value};
use crate::configs::elastic_con;

const shards:u32 = 2;
const refresh:&str = "1s";
const compression:&str = "best_compression";
fn replicas() -> u32 {
    let es_replicas = elastic_con::get_elastic_con_config().es_replicas;
    if es_replicas.is_some() {
        return es_replicas.unwrap()
    }
    return 0;
}
pub fn defaultIndexSettings() -> Value {
    json!({
    "index": {
        "number_of_shards": shards,
        "refresh_interval": refresh,
        "number_of_replicas": replicas(),
        "codec": compression
    }
})
}
pub fn abi() -> Value {
    json!({
    "index_patterns": format!("{}{}",elastic_con::get_elastic_con_config().chain,"-abi-*"),
    "settings": defaultIndexSettings(),
    "mappings": {
        "properties": {
            "@timestamp": {"type": "date"},
            "block": {"type": "long"},
            "account": {"type": "keyword"},
            "abi": {"enabled": false},
            "abi_hex": {"enabled": false},
            "actions": {"type": "keyword"},
            "tables": {"type": "keyword"}
        }
    }
})
}