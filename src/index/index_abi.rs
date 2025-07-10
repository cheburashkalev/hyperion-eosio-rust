use crate::index::definitions::elastic_docs::AbiDocument;
use crate::{configs, elastic_hyperion, measure_time};
use elasticsearch::IndexParts;
use eosio_shipper_gf::shipper_types::AccountV0;
use log::error;
use serde_json::{Value, json};
use std::fmt::format;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use rs_abieos::Abieos;
use tokio::sync::Semaphore;

pub async fn parse_new_abi(abi_abieos: &Abieos,semaphore:Arc<Semaphore>, acc: AccountV0, block_ts: String, block_num: u32) {
    if acc.abi != "" {

        let parsed_abi = abi_abieos
            .hex_to_json("0", "abi_def", acc.abi.clone())
            .unwrap();
        //abi_abieos.destroy();
        start_async(semaphore,acc, block_ts, block_num, parsed_abi);
    }
}
fn start_async(semaphore:Arc<Semaphore>,acc: AccountV0, block_ts: String, block_num: u32, parsed_abi: String) {
    tokio::spawn(async move {
        let permit = semaphore.acquire().await.unwrap();
        if acc.abi != "" {
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
            measure_time!("Запрос на добавление ABI", {
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
            });
        }
        drop(permit);
    });
}
