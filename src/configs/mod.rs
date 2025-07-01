pub mod elastic_con;

use std::fs;
use std::io::{Read, Write};
use std::sync::OnceLock;
use serde_json::{json, Value};
use log::{log, warn};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct Loading {
    pub load_configs_from_etc: bool
}

impl Default for Loading {
    fn default() -> Self {
        Loading {
            load_configs_from_etc: true
        }
    }
}


// Lazy init
static LOADING_CONFIG: OnceLock<Loading> = OnceLock::new();
const PATH_LOADING_JSON: &str = "./configs/loading.json";
fn create_file_loading_json(config: &Loading){
    let json = serde_json::to_string_pretty(config).unwrap();
    let mut file = fs::File::create(PATH_LOADING_JSON).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
fn get_loading_config() -> &'static Loading {
    LOADING_CONFIG.get_or_init(|| {
        println!("Start load Loading file PATH: {}.",PATH_LOADING_JSON);
        let file = fs::read_to_string(PATH_LOADING_JSON);
        let config = Loading::default();
        match file {
            Ok(text) => {
                let config_raw = serde_json::from_str::<Loading>(&text);
                match config_raw {
                    Ok(config_file) => {
                        config_file
                    },
                    Err(e) => {
                        warn!("Error load file PATH: {}. Start Init from struct Loading::default(). FROM RUST LANG: {}.",PATH_LOADING_JSON,e);
                        create_file_loading_json(&config);
                        config
                    }
                }
            },
            Err(e)=>{
                warn!("Error load file PATH: {}. Start Init from struct Loading::default(). FROM RUST LANG: {}.",PATH_LOADING_JSON,e);
                create_file_loading_json(&config);
                config
            }
        }

    })
}



pub fn get_load_configs_from_etc() -> bool {
    get_loading_config().load_configs_from_etc
}