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
fn get_path_abi_json() -> String {
    let folder = configs::get_part_path_to_configs();
    let file_path = format!("{}{}",folder,FILE_NAME_ABI_JSON);
    file_path
}
fn create_file_abi_json(config: &Value){
    let json = serde_json::to_string_pretty(config).unwrap();
    let folder = configs::get_part_path_to_configs();
    fs::create_dir_all(folder).unwrap();
    let mut file = fs::File::create(get_path_abi_json()).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
pub fn get_abi_config() -> &'static String {
    let file_path = &get_path_abi_json();
    let config = get_def_abi_json();
    ABI_CONFIG.get_or_init(|| {
        println!("Start load ABI_CONFIG file PATH: {}.",file_path);
        let file = fs::read_to_string(file_path);
        match file {
            Ok(text) => {
                let config_raw = serde_json::from_str::<Value>(&text);
                match config_raw {
                    Ok(config) => {
                        serde_json::to_string_pretty(&config).unwrap()
                    },
                    Err(e) => {
                        warn!("Error load file PATH: {}. Start Init from struct get_def_abi_json(). FROM RUST LANG: {}.",file_path,e);
                        create_file_abi_json(&config);
                        serde_json::to_string_pretty(&config).unwrap()
                    }
                }
            },
            Err(e)=>{
                warn!("Error load file PATH: {}. Start Init from struct get_def_abi_json(). FROM RUST LANG: {}.",file_path,e);
                create_file_abi_json(&config);
                serde_json::to_string_pretty(&config).unwrap()
            }
        }

    })
}