use std::error::Error;
use elasticsearch::IndexParts;
use eosio_shipper_gf::EOSIO_SYSTEM;
use eosio_shipper_gf::shipper_types::{Account, GetBlocksResultV0Ex, ShipResultsEx, SignedBlock, TableRowTypes};
use futures_util::{SinkExt, StreamExt};
use libabieos_sys::ABIEOS;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Bytes, Message};
pub mod definitions;
mod index_abi;

use crate::{configs, elastic_hyperion};
use crate::index::definitions::elastic_abi_doc::AbiDocument;
use crate::index::definitions::get_blocks_request::GetBlocksRequestV0;
use crate::index::definitions::get_status_request::{BinaryMarshaler, GetStatusRequestV0};
pub async fn start_index_block_result_v0() -> Result<(), Box<dyn Error>> {

    println!("Init Indexing");
    // Prepare configs
    println!("Init configs");
    let abi_config = configs::abi::get_abi_config();
    let ship_config = configs::ship::get_ship_con_config();

    // Init client elastic search
    println!("Init client elastic search");
    let client = elastic_hyperion::get_elastic_client().await?;

    println!("Init SHIP connection");
    let ws_stream = connect_async(ship_config.url.clone()).await?.0;
    println!("SHIP has been connected");

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // Send messages
    tokio::spawn(async move {
        while let Some(req_bytes) = rx.recv().await {
            if let Err(e) = write.send(Message::Binary(Bytes::from(req_bytes))).await {
                eprintln!("Ошибка отправки: {e}");
            }
        }
    });
    let mut next_block: u32 = 1;
    println!("Init SHIPPER_ABI");
    let mut shipper_abi: ABIEOS = ABIEOS::new();
    println!("Start listening SHIP messages");
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let msg_text = text;
                println!("NEW SHIPPER_ABI has been received");
                shipper_abi = ABIEOS::new_with_abi(EOSIO_SYSTEM, &msg_text)?;

                let status_request = GetStatusRequestV0::new();
                let request_bytes = status_request.marshal_binary()?;
                tx.send(request_bytes)?;
            },
            Ok(Message::Binary(data)) => {
                //let r = ShipResultsEx::from_bin(&shipper_abi, &data).unwrap();
                match ShipResultsEx::from_bin(&shipper_abi, &data) {
                    Ok(msg) => {
                        match msg {
                            ShipResultsEx::BlockResult(BlockResult) => {
                                let this_block = BlockResult.this_block.unwrap();
                                let block = BlockResult.block.unwrap();
                                let block_ts: String = match block {
                                    SignedBlock::signed_block_v0(_block) =>
                                        {
                                            _block.signed_header.header.timestamp
                                        },
                                    SignedBlock::signed_block_v1(_block) =>
                                        {
                                            _block.signed_header.header.timestamp
                                        }
                                };
                                let head_block = BlockResult.head;
                                let head_block_num = head_block.block_num;
                                println!("BlockResult received - Head: {}, This Block: {}, Transactions: {}, Traces: {}, Deltas: {}", head_block_num, this_block.block_num, BlockResult.transactions.len(), BlockResult.traces.len(), BlockResult.deltas.len());
                                for delta in BlockResult.deltas {
                                    if delta.name == "account"{
                                        for row in delta.rows {
                                            match row.data{
                                                TableRowTypes::account(acc) => {
                                                    match acc {
                                                        Account::account_v0(acc) => {
                                                            if acc.abi != "" {


                                                                let abi_ABIEOS: ABIEOS = ABIEOS::new_with_abi("eosio", &abi_config)?;
                                                                let parsed_abi = abi_ABIEOS.hex_to_json("eosio", "abi_def", acc.abi.as_bytes())?;

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
                                                                    block: this_block.block_num,
                                                                    abi: parsed_abi,
                                                                    abi_hex: acc.abi,
                                                                    actions: actions.clone(),
                                                                    tables: tables.clone(),
                                                                };

                                                                //// Отправка в Elasticsearch
                                                                // Настроенный клиент
                                                                let response = client
                                                                    .index(IndexParts::IndexId("gf-abi", "1"))
                                                                    .body(json!(abi_doc))
                                                                    .send()
                                                                    .await?;
                                                                println!("Response elastic: {:?}", response);
                                                            }
                                                        }
                                                    }
                                                },
                                                _ => todo!()
                                            }
                                        }
                                    }
                                }
                                if this_block.block_num % 10000 == 0 {


                                    next_block = this_block.block_num + 1;


                                    let req = GetBlocksRequestV0 {
                                        start_block_num: next_block,
                                        end_block_num: head_block_num,
                                        max_messages_in_flight: 10000,
                                        have_positions: vec![],
                                        irreversible_only: false,
                                        fetch_block: true,
                                        fetch_traces: true,
                                        fetch_deltas: true,
                                    };

                                    if let Ok(req_bytes) = req.marshal_binary() {
                                        tx.send(req_bytes)?;
                                    }
                                }
                            },
                            // Get new Status from NodeOS EOSIO
                            ShipResultsEx::Status(status) => {
                                println!("Status received - Head: {}, Last irreversible: {}",
                                         status.head.block_num, status.last_irreversible.block_num);
                                // Request first 10000 blocks
                                let req = GetBlocksRequestV0 {
                                    start_block_num: next_block,
                                    end_block_num: status.head.block_num,
                                    max_messages_in_flight: 10000,
                                    have_positions: vec![],
                                    irreversible_only: false,
                                    fetch_block: true,
                                    fetch_traces: true,
                                    fetch_deltas: true,
                                };

                                if let Ok(req_bytes) = req.marshal_binary() {
                                    tx.send(req_bytes)?;
                                }
                            }
                        }
                    },
                    Err(_) => todo!(),
                }
                continue;
            }
            Ok(Message::Ping(_)) => println!("Ping received"),
            Ok(Message::Pong(_)) => println!("Pong received"),
            Ok(Message::Close(_)) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    Ok(())
}
pub fn index_block_result_v0(block_res: GetBlocksResultV0Ex)
{

}