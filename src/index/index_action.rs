use crate::index::definitions::elastic_docs::{
    AbiDocument, ActionDocument, ActionReceipt, BlockDocument, HyperionActionAct,
};
use crate::{configs, elastic_hyperion, measure_time};
use elasticsearch::{IndexParts, SearchParts};
use eosio_shipper_gf::EOSIO_SYSTEM;
use eosio_shipper_gf::shipper_types::{AccountAuthSequence, AccountDelta, ActionReceiptVariant, ActionTraceVariant, BlockHeader, BlockPosition, PartialTransactionVariant, ProducerKey, ProducerSchedule, PrunableData, SignedBlock, Traces, Transaction, TransactionReceiptV0, TransactionTraceV0};
use futures_util::TryFutureExt;
use libabieos_sys::ABIEOS;
use log::{error, trace};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::{Value, from_str, json};
use std::fmt::format;
use std::ops::Deref;
use std::string::String;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tokio::sync::Semaphore;

// Функция для обработки экранированного JSON
fn parse_json(s: &str) -> Result<Value, serde_json::Error> {
    match from_str::<Value>(s) {
        Ok(Value::String(inner)) => from_str(&inner), // Двойное деэкранирование
        Ok(v) => Ok(v),                               // Уже валидный JSON
        Err(_) => from_str(s),                        // Прямая десериализация (если не сработало)
    }
}
pub async fn parse_new_action(
    semaphore: Arc<Semaphore>,
    header: &BlockHeader,
    traces: Vec<Traces>,
    block_ts: &String,
    this_block: &BlockPosition,
) {
    for ref_traces in traces {
        match ref_traces {
            Traces::transaction_trace_v0(t) => {
                if t.partial.iter().len() > 0 {
                    let ref_partial = t.partial.as_ref();
                    if ref_partial.is_none() {
                        continue;
                    }
                    let signatures = match ref_partial.unwrap() {
                        PartialTransactionVariant::partial_transaction_v0(e) => {
                            Some(e.signatures.clone())
                        }
                        PartialTransactionVariant::partial_transaction_v1(e) => {
                            let ref_prunable_data = e.prunable_data.as_ref();
                            if ref_prunable_data.is_none() {
                                None
                            } else {
                                match ref_prunable_data.unwrap() {
                                    PrunableData::prunable_data_full_legacy(x) => {
                                        Some(x.signatures.clone())
                                    }
                                    PrunableData::prunable_data_none(x) => None,
                                    PrunableData::prunable_data_partial(x) => {
                                        Some(x.signatures.clone())
                                    }
                                    PrunableData::prunable_data_full(x) => {
                                        Some(x.signatures.clone())
                                    }
                                }
                            }
                        }
                    };
                    let account_ram_delta = t.account_ram_delta.as_ref();
                    let mut account_ram_deltas: Option<Vec<Value>> = None;
                    if account_ram_delta.is_some() {
                        let ard = account_ram_delta.unwrap();
                        account_ram_deltas = Some(vec![json!(AccountDelta{
                            account: ard.account.clone(),
                            delta: ard.delta.clone()
                        })]);
                    }
                    let block_num = this_block.block_num.clone();
                    let block_id = this_block.block_id.clone();
                    let timestamp = block_ts.clone();
                    let producer = header.producer.clone();
                    let trx_id = t.id.clone();
                    let mut elapsed: Option<String> = None;
                    let clone_elapsed = t.elapsed.clone();
                    if !clone_elapsed.eq("0") {
                        elapsed = Some(clone_elapsed);
                    }
                    let mut except: Option<String> = None;
                    if t.except.is_some() {
                        except = Some(t.except.as_ref().unwrap().clone());
                    }
                    let cpu_usage_us = Some(t.cpu_usage_us);
                    let net_usage_words = Some(t.net_usage_words);
                    let inline_count = Some(t.action_traces.len() as u32);
                    let mut docs: Vec<ActionDocument> = vec![];
                    for ref_action in t.action_traces {
                        let mut global_sequence = String::new();
                        let mut context_free = None;
                        let mut receipts = Vec::new();
                        let mut creator_action_ordinal: u32 = 0;
                        let mut action_ordinal: u32 = 0;
                        let mut act_digest = String::new();
                        let mut act = HyperionActionAct::default();
                        match ref_action {
                            ActionTraceVariant::action_trace_v0(a) => {
                                let act_authorization = a.act.authorization.clone();
                                let client = elastic_hyperion::get_elastic_client().await.unwrap();
                                let response = client
                                    .search(SearchParts::Index(&["gf-abi*"]))
                                    .body(json!({
                            "query": {
                                "bool": {
                                    "must": [
                                        { "range": { "block": { "lte": this_block.block_num.clone() } } },
                                        { "term": { "account": a.act.account.clone() } }
                                    ]
                                }
                            },
                            "size": 1, // Лимит результатов
                            "_source": ["abi"], // Возвращаем только поле abi
                            "sort": [{ "block": {"order":"desc"} }] // Сортируем по block в убывающем порядке
                        }))
                                    .send().await.unwrap();

                                // Парсим ответ
                                let response_body = response.json::<Value>().await.unwrap();
                                //println!("{:?}", response_body.to_string());
                                let hits = response_body["hits"]["hits"].as_array().unwrap();
                                let mut abi =
                                    configs::abi_eosio::get_abi_eosio_config().to_string();
                                if hits.len() != 0 {
                                    for hit in hits {
                                        let abi_s = &hit["_source"]["abi"];
                                        abi = abi_s.to_string();
                                    }
                                }
                                //println!("ABI: ...  {}  ...", abi);
                                abi = parse_json(abi.as_str()).unwrap().to_string();
                                //abi = abi.replace("\\\"","\"").replace("\\n","").replace("","");

                                let shipper_abi = ABIEOS::new_with_abi(a.act.account.as_str(), abi.as_str()).unwrap_or_else(|e|{
                                  panic!("Error create shipper abi: {:?}", e);
                                });
                                let hex = a.act.data.as_bytes();
                                let hex = hex::decode(a.act.data.as_str()).unwrap();
                                let hex = hex.as_slice();
                                let data = shipper_abi
                                    .bin_to_json(a.act.account.as_str(), a.act.name.as_str(), hex)
                                    .unwrap_or_else(|e|{
                                        shipper_abi.destroy();
                                    panic!("Error parse data action: {:?}", e);
                                });
                                shipper_abi.destroy();
                                let data = parse_json(data.as_str()).unwrap();
                                
                                act = HyperionActionAct {
                                    name: a.act.name.clone(),
                                    account: a.act.account.clone(),
                                    authorization: act_authorization,
                                    data,
                                };

                                let ref_receipt = a.receipt.as_ref();
                                let clone_context_free = a.context_free.clone();
                                creator_action_ordinal = a.creator_action_ordinal;
                                action_ordinal = a.action_ordinal;
                                if clone_context_free != false {
                                    context_free = Some(clone_context_free);
                                }
                                if ref_receipt.is_some() {
                                    match ref_receipt.clone().unwrap() {
                                        ActionReceiptVariant::action_receipt_v0(r) => {
                                            let mut auth_sequence: Vec<AccountAuthSequence> =
                                                Vec::new();
                                            for auth in &r.auth_sequence {
                                                auth_sequence.push(AccountAuthSequence {
                                                    account: auth.account.clone(),
                                                    sequence: auth.sequence.clone(),
                                                });
                                            }
                                            act_digest = r.act_digest.clone();
                                            global_sequence = r.global_sequence.clone();
                                            let receiver = r.receiver.clone();
                                            let recv_sequence = r.recv_sequence.clone();
                                            receipts.push(ActionReceipt {
                                                global_sequence: r.global_sequence.clone(),
                                                auth_sequence,
                                                receiver,
                                                recv_sequence,
                                            });
                                        }
                                    }
                                }
                            }
                            ActionTraceVariant::action_trace_v1(a) =>{
                                let act_authorization = a.act.authorization.clone();
                                let client = elastic_hyperion::get_elastic_client().await.unwrap();
                                let response = client
                                    .search(SearchParts::Index(&["gf-abi*"]))
                                    .body(json!({
                            "query": {
                                "bool": {
                                    "must": [
                                        { "range": { "block": { "lte": this_block.block_num.clone() } } },
                                        { "term": { "account": a.act.account.clone() } }
                                    ]
                                }
                            },
                            "size": 1, // Лимит результатов
                            "_source": ["abi"], // Возвращаем только поле abi
                            "sort": [{ "block": {"order":"desc"} }] // Сортируем по block в убывающем порядке
                        }))
                                    .send().await.unwrap();

                                // Парсим ответ
                                let response_body = response.json::<Value>().await.unwrap();
                                //println!("{:?}", response_body.to_string());
                                let hits = response_body["hits"]["hits"].as_array().unwrap();
                                let mut abi =
                                    configs::abi_eosio::get_abi_eosio_config().to_string();
                                if hits.len() != 0 {
                                    for hit in hits {
                                        let abi_s = &hit["_source"]["abi"];
                                        abi = abi_s.to_string();
                                    }
                                }
                                //println!("ABI: ...  {}  ...", abi);
                                abi = parse_json(abi.as_str()).unwrap().to_string();
                                //abi = abi.replace("\\\"","\"").replace("\\n","").replace("","");

                                let shipper_abi =
                                    ABIEOS::new_with_abi(&*a.act.account.clone(), abi.as_str())
                                        .unwrap();
                                let hex = a.act.data.as_bytes();
                                
                                let data = shipper_abi
                                    .hex_to_json(&*a.act.account.clone(), a.act.name.as_str(), hex)
                                    .unwrap();
                                shipper_abi.destroy();
                                let data = parse_json(data.as_str()).unwrap().to_string();
                                
                                act = HyperionActionAct {
                                    name: a.act.name.clone(),
                                    account: a.act.account.clone(),
                                    authorization: act_authorization,
                                    data: Value::from(data),
                                };

                                let ref_receipt = a.receipt.as_ref();
                                let clone_context_free = a.context_free.clone();
                                creator_action_ordinal = a.creator_action_ordinal;
                                action_ordinal = a.action_ordinal;
                                if clone_context_free != false {
                                    context_free = Some(clone_context_free);
                                }
                                if ref_receipt.is_some() {
                                    match ref_receipt.clone().unwrap() {
                                        ActionReceiptVariant::action_receipt_v0(r) => {
                                            let mut auth_sequence: Vec<AccountAuthSequence> =
                                                Vec::new();
                                            for auth in &r.auth_sequence {
                                                auth_sequence.push(AccountAuthSequence {
                                                    account: auth.account.clone(),
                                                    sequence: auth.sequence.clone(),
                                                });
                                            }
                                            act_digest = r.act_digest.clone();
                                            global_sequence = r.global_sequence.clone();
                                            let receiver = r.receiver.clone();
                                            let recv_sequence = r.recv_sequence.clone();
                                            receipts.push(ActionReceipt {
                                                global_sequence: r.global_sequence.clone(),
                                                auth_sequence,
                                                receiver,
                                                recv_sequence,
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        //let new_producers = <ProducerSchedule as Clone>::clone(&header.new_producers.clone().unwrap());
                        let account_ram_deltas = account_ram_deltas.clone();
                        let block_id = block_id.clone();
                        let elapsed = elapsed.clone();
                        let except = except.clone();
                        let producer = producer.clone();

                        let signatures = signatures.clone();
                        let timestamp = timestamp.clone();
                        let trx_id = trx_id.clone();
                        docs.push(ActionDocument {
                            timestamp,
                            signatures,
                            act,
                            block_num,
                            block_id,
                            global_sequence,
                            producer,
                            trx_id,
                            account_ram_deltas,
                            elapsed,
                            context_free,
                            except,
                            receipts,
                            creator_action_ordinal,
                            action_ordinal,
                            cpu_usage_us,
                            net_usage_words,
                            inline_count,
                            inline_filtered: false,
                            act_digest,
                        });
                    }
                    start_async(semaphore.clone(), docs);
                }
            }
        }
    }
}
fn start_async(semaphore: Arc<Semaphore>, docs: Vec<ActionDocument>) {
    let docs_clone = docs;
    tokio::spawn(async move {
        let permit = semaphore.acquire().await.unwrap();
        for block_doc in docs_clone {
            let response = elastic_hyperion::get_elastic_client()
                .await
                .unwrap()
                .index(IndexParts::IndexId(
                    "gf-action",
                    &*block_doc.global_sequence.clone(),
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
        }
        drop(permit);
    });
}
