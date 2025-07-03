use std::fs;
use std::io::{Write};
use std::sync::OnceLock;
use log::{warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::configs;
use crate::configs::{PATH_CONFIGS_JSON, PATH_WORKDIR};

fn get_def_abi_json() -> Value {
    json!({
    "version": "eosio::abi/1.1",
    "structs": [
        {
            "name": "extensions_entry",
            "base": "",
            "fields": [
                {
                    "name": "tag",
                    "type": "uint16"
                },
                {
                    "name": "value",
                    "type": "bytes"
                }
            ]
        },
        {
            "name": "type_def",
            "base": "",
            "fields": [
                {
                    "name": "new_type_name",
                    "type": "string"
                },
                {
                    "name": "type",
                    "type": "string"
                }
            ]
        },
        {
            "name": "field_def",
            "base": "",
            "fields": [
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "type",
                    "type": "string"
                }
            ]
        },
        {
            "name": "struct_def",
            "base": "",
            "fields": [
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "base",
                    "type": "string"
                },
                {
                    "name": "fields",
                    "type": "field_def[]"
                }
            ]
        },
        {
            "name": "action_def",
            "base": "",
            "fields": [
                {
                    "name": "name",
                    "type": "name"
                },
                {
                    "name": "type",
                    "type": "string"
                },
                {
                    "name": "ricardian_contract",
                    "type": "string"
                }
            ]
        },
        {
            "name": "table_def",
            "base": "",
            "fields": [
                {
                    "name": "name",
                    "type": "name"
                },
                {
                    "name": "index_type",
                    "type": "string"
                },
                {
                    "name": "key_names",
                    "type": "string[]"
                },
                {
                    "name": "key_types",
                    "type": "string[]"
                },
                {
                    "name": "type",
                    "type": "string"
                }
            ]
        },
        {
            "name": "clause_pair",
            "base": "",
            "fields": [
                {
                    "name": "id",
                    "type": "string"
                },
                {
                    "name": "body",
                    "type": "string"
                }
            ]
        },
        {
            "name": "error_message",
            "base": "",
            "fields": [
                {
                    "name": "error_code",
                    "type": "uint64"
                },
                {
                    "name": "error_msg",
                    "type": "string"
                }
            ]
        },
        {
            "name": "variant_def",
            "base": "",
            "fields": [
                {
                    "name": "name",
                    "type": "string"
                },
                {
                    "name": "types",
                    "type": "string[]"
                }
            ]
        },
        {
            "name": "abi_def",
            "base": "",
            "fields": [
                {
                    "name": "version",
                    "type": "string"
                },
                {
                    "name": "types",
                    "type": "type_def[]"
                },
                {
                    "name": "structs",
                    "type": "struct_def[]"
                },
                {
                    "name": "actions",
                    "type": "action_def[]"
                },
                {
                    "name": "tables",
                    "type": "table_def[]"
                },
                {
                    "name": "ricardian_clauses",
                    "type": "clause_pair[]"
                },
                {
                    "name": "error_messages",
                    "type": "error_message[]"
                },
                {
                    "name": "abi_extensions",
                    "type": "extensions_entry[]"
                },
                {
                    "name": "variants",
                    "type": "variant_def[]$"
                }
            ]
        }
    ]
})
}
static ABI_CONFIG: OnceLock<String> = OnceLock::new();
const FILE_NAME_ABI_JSON: &str = "abi.json";

pub fn get_abi_config() -> &'static String {
    ABI_CONFIG.get_or_init(|| {
        println!("Start loading \'ABI_CONFIG\' file: {}.",FILE_NAME_ABI_JSON);
        let config = configs::load_configs_json(FILE_NAME_ABI_JSON,get_def_abi_json());
        config.to_string()
    })
}