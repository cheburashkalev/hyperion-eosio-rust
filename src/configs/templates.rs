use crate::configs::elastic_con;
use serde_json::{Value, json};
use std::collections::HashMap;
use serde::{Deserialize, Deserializer};

const shards: u32 = 2;
const refresh: &str = "1s";
const compression: &str = "best_compression";
fn replicas() -> u32 {
    let es_replicas = elastic_con::get_elastic_con_config().es_replicas;
    if es_replicas.is_some() {
        return es_replicas.unwrap();
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
fn abi() -> Value {
    json!({
        "index_patterns": format!("{}{}",elastic_con::get_elastic_con_config().chain,"-abi*"),
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
fn actionSettings() -> Value {
    json!(
            {
        "index": {
            "codec": compression,
            "refresh_interval": refresh,
            "number_of_shards": shards * 2,
            "number_of_replicas": replicas(),
            "sort": {
                "field": "global_sequence",
                "order": "desc"
            }
        }
    }
        )
}
fn block() -> Value {
    json!({
        "index_patterns": format!("{}{}",elastic_con::get_elastic_con_config().chain,"-block*"),
        "settings": {
            "index": {
                "codec": compression,
                "number_of_shards": shards,
                "refresh_interval": refresh,
                "number_of_replicas": replicas(),
                "sort.field": "block_num",
                "sort.order": "desc"
            }
        },
        "mappings": {
            "properties": {
                "@timestamp": {"type": "date"},
                "block_num": {"type": "long"},
                "block_id": {"type": "keyword"},
                "prev_id": {"type": "keyword"},
                "producer": {"type": "keyword"},
                "new_producers.producers.block_signing_key": {"enabled": false},
                "new_producers.producers.producer_name": {"type": "keyword"},
                "new_producers.version": {"type": "long"},
                "schedule_version": {"type": "double"},
                "cpu_usage": {"type": "integer"},
                "net_usage": {"type": "integer"}
            }
        }
    })
}
fn transferProps() -> Value {
    json!({
        "properties": {
        "from": {"type": "keyword"},
        "to": {"type": "keyword"},
        "amount": {"type": "float"},
        "symbol": {"type": "keyword"},
        "memo": {"type": "text"}
    }
    })
}

fn action() -> Value {
    let mut json = format!("{{
        \"order\": 0,
        \"index_patterns\": \"{}\",
        \"settings\": {},
        \"mappings\": {{
            \"properties\": {{
                \"@timestamp\": {{\"type\": \"date\"}},
                \"ds_error\": {{\"type\": \"boolean\"}},
                \"global_sequence\": {{\"type\": \"long\"}},
                \"account_ram_deltas.delta\": {{\"type\": \"integer\"}},
                \"account_ram_deltas.account\": {{\"type\": \"keyword\"}},
                \"act.authorization.permission\": {{\"enabled\": false}},
                \"act.authorization.actor\": {{\"type\": \"keyword\"}},
                \"act.account\": {{\"type\": \"keyword\"}},
                \"act.name\": {{\"type\": \"keyword\"}},
                \"act.data\": {{\"enabled\": false}},
                \"block_num\": {{\"type\": \"long\"}},
                \"block_id\": {{\"type\": \"keyword\"}},
                \"action_ordinal\": {{\"type\": \"long\"}},
                \"creator_action_ordinal\": {{\"type\": \"long\"}},
                \"cpu_usage_us\": {{\"type\": \"integer\"}},
                \"net_usage_words\": {{\"type\": \"integer\"}},
                \"code_sequence\": {{\"type\": \"integer\"}},
                \"abi_sequence\": {{\"type\": \"integer\"}},
                \"trx_id\": {{\"type\": \"keyword\"}},
                \"producer\": {{\"type\": \"keyword\"}},
                \"signatures\": {{\"enabled\": false}},
                \"inline_count\": {{\"type\": \"short\"}},
                \"max_inline\": {{\"type\": \"short\"}},
                \"inline_filtered\": {{\"type\": \"boolean\"}},
                \"receipts\": {{
                    \"properties\": {{
                        \"global_sequence\": {{\"type\": \"long\"}},
                        \"recv_sequence\": {{\"type\": \"long\"}},
                        \"receiver\": {{\"type\": \"keyword\"}},
                        \"auth_sequence\": {{
                            \"properties\": {{
                                \"account\": {{\"type\": \"keyword\"}},
                                \"sequence\": {{\"type\": \"long\"}}
                            }}
                        }}
                    }}
                }},
                \"@newaccount\": {{
                    \"properties\": {{
                        \"active\": {{\"type\": \"object\"}},
                        \"owner\": {{\"type\": \"object\"}},
                        \"newact\": {{\"type\": \"keyword\"}}
                    }}
                }},
                \"@updateauth\": {{
                    \"properties\": {{
                        \"permission\": {{\"type\": \"keyword\"}},
                        \"parent\": {{\"type\": \"keyword\"}},
                        \"auth\": {{\"type\": \"object\"}}
                    }}
                }},

                \"@transfer\": {},

                \"@unstaketorex\": {{
                    \"properties\": {{
                        \"owner\": {{\"type\": \"keyword\"}},
                        \"receiver\": {{\"type\": \"keyword\"}},
                        \"amount\": {{\"type\": \"float\"}}
                    }}
                }},

                \"@buyrex\": {{
                    \"properties\": {{
                        \"from\": {{\"type\": \"keyword\"}},
                        \"amount\": {{\"type\": \"float\"}}
                    }}
                }},

                \"@buyram\": {{
                    \"properties\": {{
                        \"payer\": {{\"type\": \"keyword\"}},
                        \"receiver\": {{\"type\": \"keyword\"}},
                        \"quant\": {{\"type\": \"float\"}}
                    }}
                }},

                \"@buyrambytes\": {{
                    \"properties\": {{
                        \"payer\": {{\"type\": \"keyword\"}},
                        \"receiver\": {{\"type\": \"keyword\"}},
                        \"bytes\": {{\"type\": \"long\"}}
                    }}
                }},

                \"@delegatebw\": {{
                    \"properties\": {{
                        \"from\": {{\"type\": \"keyword\"}},
                        \"receiver\": {{\"type\": \"keyword\"}},
                        \"stake_cpu_quantity\": {{\"type\": \"float\"}},
                        \"stake_net_quantity\": {{\"type\": \"float\"}},
                        \"transfer\": {{\"type\": \"boolean\"}},
                        \"amount\": {{\"type\": \"float\"}}
                    }}
                }},

                \"@undelegatebw\": {{
                    \"properties\": {{
                        \"from\": {{\"type\": \"keyword\"}},
                        \"receiver\": {{\"type\": \"keyword\"}},
                        \"unstake_cpu_quantity\": {{\"type\": \"float\"}},
                        \"unstake_net_quantity\": {{\"type\": \"float\"}},
                        \"amount\": {{\"type\": \"float\"}}
                    }}
                }}
            }}
        }}
    }}",format!("{}{}",elastic_con::get_elastic_con_config().chain,"-action*"),actionSettings(),transferProps());
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    deserializer.disable_recursion_limit();
    let deserializer = serde_stacker::Deserializer::new(&mut deserializer);
    return Value::deserialize(deserializer).unwrap();
    //let value = Value::deserialize(deserializer).unwrap();
    //json!(
    //       {
    //    "order": 0,
    //    "index_patterns": format!("{}{}",elastic_con::get_elastic_con_config().chain,"-action*"),
    //    "settings": actionSettings(),
    //    "mappings": {
    //        "properties": {
    //            "@timestamp": {"type": "date"},
    //            "ds_error": {"type": "boolean"},
    //            "global_sequence": {"type": "long"},
    //            "account_ram_deltas.delta": {"type": "integer"},
    //            "account_ram_deltas.account": {"type": "keyword"},
    //            "act.authorization.permission": {"enabled": false},
    //            "act.authorization.actor": {"type": "keyword"},
    //            "act.account": {"type": "keyword"},
    //            "act.name": {"type": "keyword"},
    //            "act.data": {"enabled": false},
    //            "block_num": {"type": "long"},
    //            "block_id": {"type": "keyword"},
    //            "action_ordinal": {"type": "long"},
    //            "creator_action_ordinal": {"type": "long"},
    //            "cpu_usage_us": {"type": "integer"},
    //            "net_usage_words": {"type": "integer"},
    //            "code_sequence": {"type": "integer"},
    //            "abi_sequence": {"type": "integer"},
    //            "trx_id": {"type": "keyword"},
    //            "producer": {"type": "keyword"},
    //            "signatures": {"enabled": false},
    //            "inline_count": {"type": "short"},
    //            "max_inline": {"type": "short"},
    //            "inline_filtered": {"type": "boolean"},
    //            "receipts": {
    //                "properties": {
    //                    "global_sequence": {"type": "long"},
    //                    "recv_sequence": {"type": "long"},
    //                    "receiver": {"type": "keyword"},
    //                    "auth_sequence": {
    //                        "properties": {
    //                            "account": {"type": "keyword"},
    //                            "sequence": {"type": "long"}
    //                        }
    //                    }
    //                }
    //            },
    //            "@newaccount": {
    //                "properties": {
    //                    "active": {"type": "object"},
    //                    "owner": {"type": "object"},
    //                    "newact": {"type": "keyword"}
    //                }
    //            },
    //            "@updateauth": {
    //                "properties": {
    //                    "permission": {"type": "keyword"},
    //                    "parent": {"type": "keyword"},
    //                    "auth": {"type": "object"}
    //                }
    //            },
//
    //            "@transfer": transferProps(),
//
    //            "@unstaketorex": {
    //                "properties": {
    //                    "owner": {"type": "keyword"},
    //                    "receiver": {"type": "keyword"},
    //                    "amount": {"type": "float"}
    //                }
    //            },
//
    //            "@buyrex": {
    //                "properties": {
    //                    "from": {"type": "keyword"},
    //                    "amount": {"type": "float"}
    //                }
    //            },
//
    //            "@buyram": {
    //                "properties": {
    //                    "payer": {"type": "keyword"},
    //                    "receiver": {"type": "keyword"},
    //                    "quant": {"type": "float"}
    //                }
    //            },
//
    //            "@buyrambytes": {
    //                "properties": {
    //                    "payer": {"type": "keyword"},
    //                    "receiver": {"type": "keyword"},
    //                    "bytes": {"type": "long"}
    //                }
    //            },
//
    //            "@delegatebw": {
    //                "properties": {
    //                    "from": {"type": "keyword"},
    //                    "receiver": {"type": "keyword"},
    //                    "stake_cpu_quantity": {"type": "float"},
    //                    "stake_net_quantity": {"type": "float"},
    //                    "transfer": {"type": "boolean"},
    //                    "amount": {"type": "float"}
    //                }
    //            },
//
    //            "@undelegatebw": {
    //                "properties": {
    //                    "from": {"type": "keyword"},
    //                    "receiver": {"type": "keyword"},
    //                    "unstake_cpu_quantity": {"type": "float"},
    //                    "unstake_net_quantity": {"type": "float"},
    //                    "amount": {"type": "float"}
    //                }
    //            }
    //        }
    //    }
    //});
}
pub fn templates() -> HashMap<String, Value> {
    let mut scores = HashMap::new();
    let prefix = &elastic_con::get_elastic_con_config().chain;
    scores.insert(format!("{}-block",prefix), block());
    scores.insert(format!("{}-abi",prefix), abi());
    scores.insert(format!("{}-action",prefix), action());
    scores
}
