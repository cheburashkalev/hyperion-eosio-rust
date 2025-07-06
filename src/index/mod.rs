use elasticsearch::IndexParts;
use eosio_shipper_gf::EOSIO_SYSTEM;
use eosio_shipper_gf::shipper_types::{
    Account, GetBlocksResultV0Ex, ShipResultsEx, SignedBlock, TableRowTypes,
};
use futures_util::{SinkExt, StreamExt};
use libabieos_sys::ABIEOS;
use serde_json::{Value, json};
use std::error::Error;
use std::sync::Arc;
use std::thread;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Semaphore};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Bytes, Message, Utf8Bytes};
pub mod definitions;
mod index_abi;
mod index_block;
mod index_action;

use crate::index::definitions::elastic_docs::AbiDocument;
use crate::index::definitions::get_blocks_request::GetBlocksRequestV0;
use crate::index::definitions::get_status_request::{BinaryMarshaler, GetStatusRequestV0};
use crate::{configs, elastic_hyperion};
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
    //let mut shipper_abi: ABIEOS = ABIEOS::new();
    println!("Start listening SHIP messages");
    let mut msg_texts: Utf8Bytes = Utf8Bytes::default();
    let semaphore = Arc::new(Semaphore::new(100));
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                msg_texts = text;
                println!("NEW SHIPPER_ABI has been received");

                let status_request = GetStatusRequestV0::new();
                let request_bytes = status_request.marshal_binary()?;
                tx.send(request_bytes)?;
            }
            Ok(Message::Binary(data)) => {
                let shipper_abi = ABIEOS::new_with_abi(EOSIO_SYSTEM, &msg_texts).unwrap();
                //tokio::spawn(async{

                let r = ShipResultsEx::from_bin(&shipper_abi, &data);
                shipper_abi.destroy();
                match r {
                    Ok(msg) => {
                        match msg {
                            ShipResultsEx::BlockResult(BlockResult) => {

                                let this_block = &BlockResult.this_block.unwrap();

                                let block = &BlockResult.block.unwrap();

                                let block_ts: &String = match block {
                                    SignedBlock::signed_block_v0(_block) => {
                                        &_block.signed_header.header.timestamp
                                    }
                                    SignedBlock::signed_block_v1(_block) => {
                                        &_block.signed_header.header.timestamp
                                    }
                                };
                                index_block::parse_new_block(semaphore.clone(),block, block_ts, this_block).await;
                                let head_block = BlockResult.head;
                                let head_block_num = head_block.block_num;
                                if BlockResult.transactions.len() > 0 {
                                    println!("Transactions received");
                                }
                                draw_progress_bar(this_block.block_num, head_block_num).await;
                                //println!("BlockResult received - Head: {}, This Block: {}, Transactions: {}, Traces: {}, Deltas: {}", head_block_num, this_block.block_num, BlockResult.transactions.len(), BlockResult.traces.len(), BlockResult.deltas.len());
                                for delta in BlockResult.deltas {
                                    if delta.name == "account" {
                                        for row in delta.rows {
                                            match row.data {
                                                TableRowTypes::account(acc) => match acc {
                                                    Account::account_v0(acc) => {
                                                        index_abi::parse_new_abi(
                                                            semaphore.clone(),
                                                            acc,
                                                            block_ts.to_string(),
                                                            this_block.block_num,
                                                        )
                                                        .await;
                                                    }
                                                },
                                                _ => todo!(),
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
                                        tx.send(req_bytes).unwrap();
                                    }
                                }
                            }
                            // Get new Status from NodeOS EOSIO
                            ShipResultsEx::Status(status) => {
                                println!(
                                    "Status received - Head: {}, Last irreversible: {}",
                                    status.head.block_num, status.last_irreversible.block_num
                                );
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
                                    tx.send(req_bytes).unwrap();
                                }
                            }
                        }
                    }
                    Err(_) => todo!(),
                }

                //});

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
pub fn index_block_result_v0(block_res: GetBlocksResultV0Ex) {}

async fn draw_progress_bar(current: u32, max: u32) {
    // Получаем информацию о текущем runtime
    let handle = tokio::runtime::Handle::current();
    let alive_tasks = handle.metrics().num_alive_tasks(); // Количество рабочих потоков
    let num_workers = handle.metrics().num_workers(); // Количество рабочих потоков
    // Проверяем, чтобы max не был нулевым (во избежание деления на ноль)
    let max = max.max(1);
    let current = current.min(max);

    // Размер прогресс-бара в символах (можно настроить)
    let bar_width = 50;

    // Вычисляем процент завершения
    let percent = (current as f32 / max as f32) * 100.0;

    // Вычисляем количество заполненных символов
    let filled = (percent / 100.0 * bar_width as f32).round() as usize;

    // Создаём строку прогресс-бара
    let bar = "=".repeat(filled) + &" ".repeat(bar_width - filled);

    // Выводим прогресс-бар с информацией о потоках
    print!(
        "\r[{}] {:.1}% ({}/{}) | Alive Tasks: {}| Num Workers: {}",
        bar, percent, current, max, alive_tasks,num_workers
    );
    io::stdout().flush().await.expect("Failed to flush stdout");
}
