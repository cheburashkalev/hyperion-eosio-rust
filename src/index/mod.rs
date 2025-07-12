use elasticsearch::IndexParts;
use eosio_shipper_gf::EOSIO_SYSTEM;
use eosio_shipper_gf::shipper_types::{Account, BlockPosition, GetBlocksResultV0Ex, ShipResultsEx, SignedBlock, TableRowTypes};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::error::Error;
use std::ptr::NonNull;
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;
use crossbeam::queue::SegQueue;
use rs_abieos::{abieos_context, abieos_context_s, Abieos};
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Instant};
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
pub async fn start_index_block_result_v0(start_block: u32) -> Result<(), Box<dyn Error>> {
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
    let mut next_block: u32 = start_block;
    println!("Init SHIPPER_ABI");
    //let mut shipper_abi: ABIEOS = ABIEOS::new();
    println!("Start listening SHIP messages");
    let mut msg_texts: Utf8Bytes = Utf8Bytes::default();
    let semaphore = Arc::new(Semaphore::new(300));
    let mut durations = Vec::with_capacity(10000);
    let durations_queue = SegQueue::new();
    let abi_config: &String = configs::abi::get_abi_config();
    let shipper_abi = Abieos::new();
    shipper_abi.set_abi_json("0", abi_config.clone()).unwrap_or_else(|e|{
        shipper_abi.destroy();
        panic!("Error create shipper abi: {:?}", e);
    });
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                msg_texts = text;
                //println!("NEW SHIPPER_ABI has been received \n {:?}",msg_texts);
                shipper_abi.set_abi_json(EOSIO_SYSTEM, (&msg_texts).to_string()).unwrap_or_else(|e|{
                    shipper_abi.destroy();
                    panic!("Error create shipper abi: {:?}", e);
                });
                let status_request = GetStatusRequestV0::new();
                let request_bytes = status_request.marshal_binary()?;
                tx.send(request_bytes)?;
            }
            Ok(Message::Binary(data)) =>  unsafe {
                let start = Instant::now();

                //parse_ship(r, &shipper_abi, &data, &semaphore, durations_queue, next_block, &tx).await;
                parse_ship(
                    &shipper_abi,
                    &data,
                    &semaphore,
                    &next_block,
                    &tx).await;
                //tokio::spawn({
                //    semaphore.clone().acquire().await;
                //    parse_ship(
                //    (&msg_texts).to_string(),
                //    data.to_vec(),
                //    semaphore.clone(),
                //    &next_block,
                //    &tx)
                //});
                continue;

                durations.push(start.elapsed());
                durations_queue.push(start.elapsed());
                if(durations_queue.len() >= 10000) {
                    durations_queue.pop();
                }
                if durations.len() > 10000 {
                    durations.remove(0);
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
async fn parse_ship(shipper_abi: &Abieos, data: &[u8], semaphore: &Arc<Semaphore>, start_block: &u32, tx: &UnboundedSender<Vec<u8>>) {
    let r = ShipResultsEx::from_bin(shipper_abi, data);
    match r {
        Ok(msg) => {
            match msg {
                ShipResultsEx::BlockResult(BlockResult) => {
                    let head_block = BlockResult.head;
                    let head_block_num = head_block.block_num;
                    let op_this_block = BlockResult.this_block;
                    if op_this_block.is_none(){
                        sleep(Duration::from_secs(2)).await;
                        let req = GetBlocksRequestV0 {
                            start_block_num: head_block_num - 5,
                            end_block_num: head_block_num,
                            max_messages_in_flight: 6,
                            have_positions: vec![],
                            irreversible_only: false,
                            fetch_block: true,
                            fetch_traces: true,
                            fetch_deltas: true,
                        };

                        if let Ok(req_bytes) = req.marshal_binary() {
                            tx.send(req_bytes).unwrap();
                        }
                        return;
                    }
                    let this_block = &op_this_block.unwrap();
                    let block = &BlockResult.block.unwrap();

                    let block_ts: &String = match block {
                        SignedBlock::signed_block_v0(_block) => {
                            &_block.signed_header.header.timestamp
                        }
                        SignedBlock::signed_block_v1(_block) => {
                            &_block.signed_header.header.timestamp
                        }
                    };
                    index_block::parse_new_block(block, block_ts, this_block,&BlockResult.prev_block).await;


                    draw_progress_bar(this_block.block_num, head_block_num).await;
                    //println!("BlockResult received - Head: {}, This Block: {}, Transactions: {}, Traces: {}, Deltas: {}", head_block_num, this_block.block_num, BlockResult.transactions.len(), BlockResult.traces.len(), BlockResult.deltas.len());
                    for delta in BlockResult.deltas {
                        if delta.name == "account" {
                            for row in delta.rows {
                                match row.data {
                                    TableRowTypes::account(acc) => match acc {
                                        Account::account_v0(acc) => {
                                            index_abi::parse_new_abi(shipper_abi,
                                                                     semaphore.clone(),
                                                                     acc,
                                                                     block_ts.to_string(),
                                                                     this_block.block_num,
                                            ).await;
                                        }
                                    },
                                    _ => todo!(),
                                }
                            }
                        }
                    }
                    if BlockResult.transactions.len() > 0 {
                        let header = match block{
                            SignedBlock::signed_block_v0(b)=>{
                                &b.signed_header.header
                            },
                            SignedBlock::signed_block_v1(b)=>{
                                &b.signed_header.header
                            }
                        };
                        index_action::parse_new_action(shipper_abi, semaphore.clone(),header,BlockResult.traces,block_ts,this_block).await;
                        //println!("Transactions received");
                    }
                    if this_block.block_num % 10000 == 0 {
                        let req = GetBlocksRequestV0 {
                            start_block_num: this_block.block_num + 1,
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
                        start_block_num: *start_block,
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
        Err(e) => {
            panic!("ERROR: {:?}",e);
        } ,
    }
}
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
    //let all_dur = durations_queue.into_iter().collect::<Vec<Duration>>();
    //let total: Duration  = all_dur.iter().sum();
    //let dur_avg = total / 10000;
//
    //let dir_min = all_dur.iter().min().unwrap();
    //let dur_max = all_dur.iter().max().unwrap();
    //dur_avg.as_millis()
    // Конвертируем микросекунды в секунды (1 µs = 1e-6 секунды)
    //let seconds_per_operation = dur_avg.as_secs_f64() / 1_000_000.0;

    // Вычисляем количество операций в секунду (1 / время одной операции)
    //let operations_per_second = 1.0 / dur_avg.as_secs_f64();
    // Выводим прогресс-бар с информацией о потоках
    print!(
        "\r[{}] {:.1}% ({}/{}) | Alive Tasks: {} | Num Workers: {} | total: {:?} | agv: {:?} | min: {:?} | max: {:?} | BPS: {} |",
        bar, percent, current, max, alive_tasks,num_workers,0,0,0,0,0//operations_per_second as u32
    );
    io::stdout().flush().await.expect("Failed to flush stdout");
}
