use std::fs;
use std::io::{Write};
use std::sync::OnceLock;
use log::{warn};
use serde::{Deserialize, Serialize};
use crate::configs;

#[derive(Deserialize, Serialize, Debug)]
pub struct ShipConConfig {
    pub url: String
}
impl Default for ShipConConfig {
    fn default() -> Self {
        ShipConConfig {
            url: "ws://116.202.173.189:17777".to_string()
        }
    }
}
static SHIP_CON_CONFIG: OnceLock<ShipConConfig> = OnceLock::new();
const FILE_SHIP_CON_JSON: &str = "ship_con.json";
fn get_path_ship_con_json() -> String {
    let folder = configs::get_part_path_to_configs();
    let file_path = format!("{}{}",folder,FILE_SHIP_CON_JSON);
    file_path
}
fn create_file_ship_con_json(config: &ShipConConfig){
    let json = serde_json::to_string_pretty(config).unwrap();
    let folder = configs::get_part_path_to_configs();
    fs::create_dir_all(folder).unwrap();
    let mut file = fs::File::create(get_path_ship_con_json()).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
pub fn get_ship_con_config() -> &'static ShipConConfig {
    let file_path = &get_path_ship_con_json();
    let config = ShipConConfig::default();
    SHIP_CON_CONFIG.get_or_init(|| {
        println!("Start load SHIP_CON_CONFIG file PATH: {}.",file_path);
        let file = fs::read_to_string(file_path);
        match file {
            Ok(text) => {
                let config_raw = serde_json::from_str::<ShipConConfig>(&text);
                match config_raw {
                    Ok(config) => {
                        config
                    },
                    Err(e) => {
                        warn!("Error load file PATH: {}. Start Init from struct ShipConConfig::default(). FROM RUST LANG: {}.",file_path,e);
                        create_file_ship_con_json(&config);
                        config
                    }
                }
            },
            Err(e)=>{
                warn!("Error load file PATH: {}. Start Init from struct ShipConConfig::default(). FROM RUST LANG: {}.",file_path,e);
                create_file_ship_con_json(&config);
                config
            }
        }

    })
}