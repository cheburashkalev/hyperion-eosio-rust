use elasticsearch::IndexParts;
use eosio_shipper_gf::shipper_types::AccountV0;
use libabieos_sys::ABIEOS;
use serde_json::{json, Value};
use crate::{configs, elastic_hyperion};
use crate::index::definitions::elastic_abi_doc::AbiDocument;


pub async fn parse_new_abi(acc: AccountV0,block_ts:String,block_num: u32) {
    if acc.abi != "" {
        let abi_config: &String = configs::abi::get_abi_config();
        let abi_abieos: ABIEOS = ABIEOS::new_with_abi("eosio", &abi_config).unwrap();
        let parsed_abi = abi_abieos.hex_to_json("eosio", "abi_def", acc.abi.as_bytes()).unwrap();

        let abi_json: Value = serde_json::from_str(parsed_abi.as_str()).unwrap();
        let mut actions: Vec<String> = Vec::new();
        abi_json["actions"].as_array().unwrap().iter().for_each(|action| {
            actions.push( action["name"].as_str().unwrap().to_string());
        });
        let mut tables: Vec<String> = Vec::new();;
        abi_json["tables"].as_array().unwrap().iter().for_each(|table| {
            tables.push( table["name"].as_str().unwrap().to_string());
        });

        let abi_doc = AbiDocument {
            timestamp: block_ts.to_string(), // Конвертация в формат ISO 8601
            account: acc.name.to_string(),
            block: block_num,
            abi: parsed_abi,
            abi_hex: acc.abi,
            actions: actions.clone(),
            tables: tables.clone(),
        };

        //// Отправка в Elasticsearch
        // Настроенный клиент
        
        let response = elastic_hyperion::get_elastic_client().await.unwrap()
            .index(IndexParts::IndexId("gf-abi", block_num.to_string().as_str()))
            .body(json!(abi_doc))
            .send()
            .await.unwrap();
        println!("Response elastic: {:?}", response);
    }
}