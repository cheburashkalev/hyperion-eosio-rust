
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::fs;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, tungstenite::Bytes};
use url::Url;
use tokio::sync::mpsc;
use std::io::{Cursor, Read};
use std::panic::take_hook;
use antelope::chain::abi::{AbiTypeDef, ABI};
use antelope::chain::Packer;
use byteorder::{LittleEndian, ReadBytesExt};
use elasticsearch::{Elasticsearch, IndexParts};
use elasticsearch::auth::{ClientCertificate, Credentials};
use elasticsearch::auth::Credentials::Basic;
use elasticsearch::cert::{Certificate, CertificateValidation};
use elasticsearch::http::transport::{SingleNodeConnectionPool, Transport, TransportBuilder};
use elasticsearch::ilm::{IlmPutLifecycle, IlmPutLifecycleParts};
use eosio_shipper_gf::EOSIO_SYSTEM;
use eosio_shipper_gf::shipper_types::{Account, BlockPosition, ShipResultsEx, SignedBlock, TableRowTypes, Traces, TransactionTraceV0};
use libabieos_sys::{AbiFiles, ABIEOS};
use rs_abieos::Abieos;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::{Utf8Bytes};
mod definitions;
mod elastic_hyperion;
mod configs;
// Enum для сообщений SHIP

#[derive(Debug, Default)]
pub struct GetStatusRequestV0 {}

impl GetStatusRequestV0 {
    pub fn new() -> Self {
        GetStatusRequestV0 {}
    }
}

// Трейт для сериализации
pub trait BinaryMarshaler {
    fn marshal_binary(&self) -> Result<Vec<u8>, Box<dyn Error>>;
}

impl BinaryMarshaler for GetStatusRequestV0 {
    fn marshal_binary(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(vec![0])
    }
}

#[derive(Debug)]
pub struct GetBlocksRequestV0 {
    pub start_block_num: u32,
    pub end_block_num: u32,
    pub max_messages_in_flight: u32,
    pub have_positions: Vec<BlockPosition>,
    pub irreversible_only: bool,
    pub fetch_block: bool,
    pub fetch_traces: bool,
    pub fetch_deltas: bool,
}

impl BinaryMarshaler for GetBlocksRequestV0 {
    fn marshal_binary(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buf = Vec::new();
        buf.push(1); // Тег для GetBlocksRequestV0

        buf.extend(&self.start_block_num.to_le_bytes());
        buf.extend(&self.end_block_num.to_le_bytes());
        buf.extend(&self.max_messages_in_flight.to_le_bytes());

        // Кодирование have_positions
        encode_varuint32(self.have_positions.len() as u32, &mut buf);
        for pos in &self.have_positions {
            buf.extend(&pos.block_num.to_le_bytes());
            buf.extend(pos.block_id.as_bytes());
        }

        buf.push(self.irreversible_only as u8);
        buf.push(self.fetch_block as u8);
        buf.push(self.fetch_traces as u8);
        buf.push(self.fetch_deltas as u8);

        Ok(buf)
    }
}

fn encode_varuint32(mut v: u32, buf: &mut Vec<u8>) {
    loop {
        let mut byte = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if v == 0 {
            break;
        }
    }
}
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Serialize)]
struct AbiRequest {
    account_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AbiDocument {
    #[serde(rename = "@timestamp")]
    timestamp: String, // Формат: "2022-12-31T12:34:56.789Z"
    account: String,
    block: u32,
    abi: String,      // JSON-строка
    abi_hex: String,
    actions: Vec<String>,
    tables: Vec<String>,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = elastic_hyperion::create_elastic_client().await?;
    let url = "ws://116.202.173.189:17777";
        let ws_stream = connect_async(url).await?.0;
    println!("WebSocket соединение установлено");

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();

    // Отправка сообщений
    tokio::spawn(async move {
        while let Some(req_bytes) = rx.recv().await {
            if let Err(e) = write.send(Message::Binary(Bytes::from(req_bytes))).await {
                eprintln!("Ошибка отправки: {e}");
            }
        }
    });

    // Отправляем статус-запрос


    let mut next_block: u32 = 1;
    let tx_clone = tx.clone();
    let mut shipper_abi: ABIEOS = ABIEOS::new();
    // Обработка входящих сообщений
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                let msg_text = text;
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
                                    if BlockResult.transactions.len() > 1 {
                                        println!("BlockResult received - Head: {}, This Block: {}, Transactions: {}, Traces: {}, Deltas: {}", head_block_num, this_block.block_num, BlockResult.transactions.len(), BlockResult.traces.len(), BlockResult.deltas.len());
                                    }
                                    if this_block.block_num == 51000 {
                                        println!("BlockResult received - Head: {}, This Block: {}, Transactions: {}, Traces: {}, Deltas: {}", head_block_num, this_block.block_num, BlockResult.transactions.len(), BlockResult.traces.len(), BlockResult.deltas.len());
                                    }
                                    for delta in BlockResult.deltas {
                                        if delta.name == "account"{
                                            for row in delta.rows {
                                                match row.data{
                                                    TableRowTypes::account(acc) => {
                                                        match acc {
                                                            Account::account_v0(acc) => {
                                                                if acc.abi != "" {
                                                                    let request_body = AbiRequest {
                                                                        account_name: acc.name.to_string(),
                                                                    };
                                                                    let abi = fs::read_to_string("../abi.json").unwrap();

                                                                    let abi_ABIEOS: ABIEOS = ABIEOS::new_with_abi("eosio", &abi)?;
                                                                    let parsed_abi = abi_ABIEOS.hex_to_json("eosio", "abi_def", acc.abi.as_bytes())?;
                                                                    //println!("{:?}", parsed_abi);
                                                                    //let http_client = Client::new();
                                                                    //let response = http_client
                                                                    //    .post("https://dev-history.globalforce.io/v1/chain/get_abi")
                                                                    //    .json(&request_body)
                                                                    //    .send()
                                                                    //    .await?;

                                                                    //let status = response.status();
                                                                    //let body = response.text().await?;
                                                                    // Подготовка документа для Elasticsearch
                                                                    let abi_json: Value = serde_json::from_str(parsed_abi.as_str()).unwrap();
                                                                    let mut actions: Vec<String> = Vec::new();
                                                                    abi_json["actions"].as_array().unwrap().iter().for_each(|action| {
                                                                        actions.push( action["name"].as_str().unwrap().to_string());
                                                                    });
                                                                    let mut tables: Vec<String> = Vec::new();;
                                                                    abi_json["tables"].as_array().unwrap().iter().for_each(|table| {
                                                                        tables.push( table["name"].as_str().unwrap().to_string());
                                                                    });
                                                                    //println!("Status: {}", status);
                                                                    //println!("Response: {}", body);
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
                                            tx_clone.send(req_bytes)?;
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
                                        tx_clone.send(req_bytes)?;
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