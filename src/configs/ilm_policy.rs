use crate::configs;
use crate::configs::{PATH_CONFIGS_JSON, PATH_WORKDIR};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::sync::OnceLock;

fn get_def_ilm_policy_json() -> Value {
    json!({
        "policy": {
            "phases": {
                "hot": {
                    "min_age": "0ms",
                    "actions": {
                        "rollover": {
                            "max_size": "200gb",
                            "max_age": "60d",
                            "max_docs": 100000000
                        },
                        "set_priority": {
                            "priority": 50
                        }
                    }
                },
                "warm": {
                    "min_age": "2d",
                    "actions": {
                        "allocate": {
                            "exclude": {
                                "data": "hot"
                            }
                        },
                        "set_priority": {
                            "priority": 25
                        }
                    }
                }
            }
        }
    })
}
static ILM_POLICY_CONFIG: OnceLock<String> = OnceLock::new();
const FILE_NAME_ILM_POLICY_JSON: &str = "elastic_ilm_policy.json";
fn get_path_ilm_policy_json() -> String {
    let folder = configs::get_part_path_to_configs();
    let file_path = format!("{}{}", folder, FILE_NAME_ILM_POLICY_JSON);
    file_path
}
fn create_file_ilm_policy_json(config: &Value) {
    let json = serde_json::to_string_pretty(config).unwrap();
    let folder = configs::get_part_path_to_configs();
    fs::create_dir_all(folder).unwrap();
    let mut file = fs::File::create(get_path_ilm_policy_json()).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
pub fn get_ilm_policy_config() -> &'static String {
    let file_path = &get_path_ilm_policy_json();
    let config = get_def_ilm_policy_json();
    ILM_POLICY_CONFIG.get_or_init(|| {
        println!("Start load ILM_POLICY_CONFIG file PATH: {}.",file_path);
        let file = fs::read_to_string(file_path);
        match file {
            Ok(text) => {
                let config_raw = serde_json::from_str::<Value>(&text);
                match config_raw {
                    Ok(config) => {
                        serde_json::to_string_pretty(&config).unwrap()
                    },
                    Err(e) => {
                        warn!("Error load file PATH: {}. Start Init from struct get_def_ilm_policy_json(). FROM RUST LANG: {}.",file_path,e);
                        create_file_ilm_policy_json(&config);
                        serde_json::to_string_pretty(&config).unwrap()
                    }
                }
            },
            Err(e)=>{
                warn!("Error load file PATH: {}. Start Init from struct get_def_ilm_policy_json(). FROM RUST LANG: {}.",file_path,e);
                create_file_ilm_policy_json(&config);
                serde_json::to_string_pretty(&config).unwrap()
            }
        }

    })
}
