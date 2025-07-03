use std::sync::OnceLock;
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
pub fn get_ship_con_config() -> &'static ShipConConfig {
    SHIP_CON_CONFIG.get_or_init(|| {
        println!("Start loading \'SHIP_CON\' file: {}.", FILE_SHIP_CON_JSON);
        configs::load_configs_json(FILE_SHIP_CON_JSON, ShipConConfig::default())
    })
}