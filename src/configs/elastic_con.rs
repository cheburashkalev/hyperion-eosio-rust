use std::fs;
use std::io::{Write};
use std::sync::OnceLock;
use log::{warn};
use serde::{Deserialize, Serialize};
use crate::configs;
use crate::configs::{PATH_CONFIGS_JSON, PATH_WORKDIR};

#[derive(Deserialize, Serialize, Debug)]
pub struct ElasticConConfig {
    pub url: String,
    pub path_cert_validation: String,
    pub login: String,
    pub pass: String
}
impl Default for ElasticConConfig {
    fn default() -> Self {
        ElasticConConfig {
            url: "https://localhost:9200".to_string(),
            path_cert_validation: "/home/andrei/pki/http.crt".to_string(),
            login: "elastic".to_string(),
            pass: "rILpAx=E8ZDhA7S5OF3+".to_string(),
        }
    }
}
static ELASTIC_CON_CONFIG: OnceLock<ElasticConConfig> = OnceLock::new();
const FILE_NAME_ELASTIC_CON_JSON: &str = "elastic-con.json";
fn get_path_elastic_con_json() -> String {
    let folder = configs::get_part_path_to_configs();
    let file_path = format!("{}{}",folder,FILE_NAME_ELASTIC_CON_JSON);
    file_path
}
fn create_file_elastic_con_json(config: &ElasticConConfig){
    let json = serde_json::to_string_pretty(config).unwrap();
    let folder = configs::get_part_path_to_configs();
    fs::create_dir_all(folder).unwrap();
    let mut file = fs::File::create(get_path_elastic_con_json()).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
pub fn get_elastic_con_config() -> &'static ElasticConConfig {
    let file_path = &get_path_elastic_con_json();
    let config = ElasticConConfig::default();
    ELASTIC_CON_CONFIG.get_or_init(|| {
        println!("Start load ELASTIC_CON_CONFIG file PATH: {}.",file_path);
        let file = fs::read_to_string(file_path);
        match file {
            Ok(text) => {
                let config_raw = serde_json::from_str::<ElasticConConfig>(&text);
                match config_raw {
                    Ok(config) => {
                        config
                    },
                    Err(e) => {
                        warn!("Error load file PATH: {}. Start Init from struct ElasticConConfig::default(). FROM RUST LANG: {}.",file_path,e);
                        create_file_elastic_con_json(&config);
                        config
                    }
                }
            },
            Err(e)=>{
                warn!("Error load file PATH: {}. Start Init from struct ElasticConConfig::default(). FROM RUST LANG: {}.",file_path,e);
                create_file_elastic_con_json(&config);
                config
            }
        }

    })
}