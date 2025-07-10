use crate::index::definitions::elastic_docs::{AbiDocument, BlockDocument};
use crate::{configs, elastic_hyperion, measure_time};
use elasticsearch::IndexParts;
use eosio_shipper_gf::shipper_types::{BlockHeader, BlockPosition, ProducerKey, ProducerSchedule, SignedBlock, Transaction, TransactionReceiptV0};
use log::{error, trace};
use serde_json::{Value, json};
use std::fmt::format;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use futures_util::TryFutureExt;
use rayon::prelude::*;
use tokio::sync::Semaphore;

pub async fn parse_new_block(semaphore:&Arc<Semaphore>,block: &SignedBlock, block_ts: &String, this_block: &BlockPosition,prev_block: &Option<BlockPosition>) {
        match block{
            SignedBlock::signed_block_v0(b)=>{
                let total_cpu = b.transactions.par_iter().map(|t| t.header.cpu_usage_us).sum();
                let total_net = b.transactions.par_iter().map(|t| t.header.net_usage_words).sum();
                start_async(semaphore,total_cpu, total_net, &b.signed_header.header, block_ts, this_block,prev_block);
            },
            SignedBlock::signed_block_v1(b)=>{
                let total_cpu = b.transactions.par_iter().map(|t| t.header.cpu_usage_us).sum();
                let total_net = b.transactions.par_iter().map(|t| t.header.net_usage_words).sum();
                start_async(semaphore,total_cpu, total_net, &b.signed_header.header, block_ts, this_block,prev_block);
            },
        }
}
fn start_async(semaphore:&Arc<Semaphore>,total_cpu:u32,total_net:u32,header: &BlockHeader, block_ts: &String, this_block: &BlockPosition,prev_block: &Option<BlockPosition>) {
    let block_num = this_block.block_num.clone();
    let block_id = this_block.block_id.clone();
    let timestamp = block_ts.clone();
    let producer = header.producer.clone();
    let schedule_version = header.schedule_version.clone();
    let ref_new_producers = header.new_producers.as_ref();
    let mut new_producers: Option<ProducerSchedule> = None;
    if ref_new_producers.is_some(){
        let r = header.new_producers.as_ref().unwrap();
        let mut new_producers_parse:Vec<ProducerKey> = Vec::new();
        r.producer.iter().for_each(|e|{
            new_producers_parse.push(ProducerKey{
                name: e.name.clone(),
                public_key: e.name.clone()
            })
        });
        new_producers =  Some(ProducerSchedule{
            version: r.version,
            producer: new_producers_parse,
        });
    }
    //let new_producers = <ProducerSchedule as Clone>::clone(&header.new_producers.clone().unwrap());
    let mut prev_id = None;
    if prev_block.is_some() {
        prev_id = Some(prev_block.as_ref().unwrap().block_id.clone().to_lowercase());
    }
    let semaphore = semaphore.clone();
    tokio::spawn(async move {
        let permit = semaphore.acquire().await.unwrap();
        let block_doc = BlockDocument {
            timestamp,
            cpu_usage: total_cpu,
            net_usage: total_net,
            block_num,
            block_id: block_id.to_lowercase(),
            schedule_version,
            producer,
            new_producers,
            prev_id
        };
        measure_time!("Запрос на добавление ABI", {
                let response = elastic_hyperion::get_elastic_client()
                    .await
                    .unwrap()
                    .index(IndexParts::IndexId(
                        "gf-block",
                        block_doc.block_num.to_string().as_str(),
                    ))
                    .body(json!(block_doc))
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
        drop(permit);
    });
}
