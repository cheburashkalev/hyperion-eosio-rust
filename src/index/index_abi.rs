use crate::index::definitions::elastic_docs::AbiDocument;
use crate::{configs, elastic_hyperion, measure_time};
use elasticsearch::IndexParts;
use eosio_shipper_gf::shipper_types::AccountV0;
use log::error;
use rs_abieos_gf::Abieos;
use serde_json::{Value, json};
use std::fmt::format;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tokio::sync::Semaphore;

pub async fn parse_new_abi(
    abi_abieos: &Abieos,
    acc: &AccountV0,
    block_ts: String,
    block_num: u32,
) {
    if acc.abi != "" {
        let parsed_abi = abi_abieos
            .hex_to_json("0", "abi_def", acc.abi.clone())
            .unwrap();
        //abi_abieos.destroy();
        let abi_json: Value = serde_json::from_str(parsed_abi.as_str()).unwrap();
        let mut actions: Vec<String> = Vec::new();
        abi_json["actions"]
            .as_array()
            .unwrap()
            .iter()
            .for_each(|action| {
                actions.push(action["name"].as_str().unwrap().to_string());
            });
        let mut tables: Vec<String> = Vec::new();
        abi_json["tables"]
            .as_array()
            .unwrap()
            .iter()
            .for_each(|table| {
                tables.push(table["name"].as_str().unwrap().to_string());
            });

        let abi_doc = AbiDocument {
            timestamp: block_ts, // Конвертация в формат ISO 8601
            account: acc.name.clone(),
            block: block_num,
            abi: parsed_abi,
            abi_hex: acc.abi.clone(),
            actions,
            tables,
        };

        //// Отправка в Elasticsearch
        // Настроенный клиент
        let response = elastic_hyperion::get_elastic_client()
            .await
            .unwrap()
            .index(IndexParts::IndexId(
                "gf-abi",
                format!("{}{}", block_num, acc.name).as_str(),
            ))
            .body(json!(abi_doc))
            .send()
            .await
            .unwrap();
        match response.error_for_status_code() {
            Ok(r) => {
                //println!("Response elastic: {:?}", r);
            }
            Err(e) => {
                panic!("Error elastic: {:?}", e);
            }
        }
    }
}
