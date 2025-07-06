use crate::index::definitions::elastic_docs::{
    AbiDocument, ActionDocument, ActionReceipt, BlockDocument, HyperionActionAct,
};
use crate::{configs, elastic_hyperion, measure_time};
use elasticsearch::{IndexParts, SearchParts};
use eosio_shipper_gf::EOSIO_SYSTEM;
use eosio_shipper_gf::shipper_types::{
    AccountAuthSequence, ActionReceiptVariant, ActionTraceVariant, BlockHeader, BlockPosition,
    PartialTransactionVariant, ProducerKey, ProducerSchedule, PrunableData, SignedBlock, Traces,
    Transaction, TransactionReceiptV0, TransactionTraceV0,
};
use futures_util::TryFutureExt;
use libabieos_sys::ABIEOS;
use log::{error, trace};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::{Value, json};
use std::fmt::format;
use std::ops::Deref;
use std::string::String;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use tokio::sync::Semaphore;

pub async fn parse_new_action(
    semaphore: Arc<Semaphore>,
    block: &SignedBlock,
    block_ts: &String,
    this_block: &BlockPosition,
    transactions: Vec<Option<Transaction>>,
) {

}
fn start_async(
    semaphore: Arc<Semaphore>,
    header: &BlockHeader,
    trace: &TransactionTraceV0,
    block_ts: &String,
    this_block: &BlockPosition,
) {
    if trace.partial.iter().len() > 0 {
        let ref_partial = trace.partial.as_ref();
        if ref_partial.is_none() {
            return;
        }
        let signatures = match ref_partial.unwrap() {
            PartialTransactionVariant::partial_transaction_v0(e) => Some(e.signatures.clone()),
            PartialTransactionVariant::partial_transaction_v1(e) => {
                let ref_prunable_data = e.prunable_data.as_ref();
                let empty: Vec<String> = Vec::new();
                if ref_prunable_data.is_none() {
                    None
                } else {
                    match ref_prunable_data.unwrap() {
                        PrunableData::prunable_data_full_legacy(x) => Some(x.signatures.clone()),
                        PrunableData::prunable_data_none(x) => None,
                        PrunableData::prunable_data_partial(x) => Some(x.signatures.clone()),
                        PrunableData::prunable_data_full(x) => Some(x.signatures.clone()),
                    }
                }
            }
        };
        let account_ram_delta = trace.account_ram_delta.as_ref();
        let mut account_ram_deltas: Option<Vec<Value>> = None;
        if account_ram_delta.is_some() {
            account_ram_deltas = Some(vec![json!(account_ram_delta.unwrap())]);
        }
        let block_num = this_block.block_num.clone();
        let block_id = this_block.block_id.clone();
        let timestamp = block_ts.clone();
        let producer = header.producer.clone();
        let trx_id = trace.id.clone();
        let mut elapsed: Option<String> = None;
        let clone_elapsed = trace.elapsed.clone();
        if !clone_elapsed.eq("0") {
            elapsed = Some(clone_elapsed);
        }
        let mut except: Option<String> = None;
        if trace.except.is_some() {
            except = Some(trace.except.as_ref().unwrap().clone());
        }
        let mut cpu_usage_us = Some(trace.cpu_usage_us);
        let mut net_usage_words = Some(trace.net_usage_words);
        let mut inline_count = Some(trace.action_traces.len() as u32);
        trace.action_traces.par_iter().for_each(|ref_action: &ActionTraceVariant| {
            // Создаем рантайм
            let rt = tokio::runtime::Runtime::new().unwrap();

            // Блокируем текущий поток и выполняем асинхронный код
            
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
                    let client = rt.block_on(elastic_hyperion::get_elastic_client()).unwrap();
                    let response = rt.block_on(client
                        .search(SearchParts::Index(&["gf-abi-v1"]))
                        .body(json!({
                            "query": {
                                "bool": {
                                    "must": [
                                        { "range": { "block": { "lte": this_block.block_num.clone() } } },
                                        { "term": { "account.keyword": a.act.account.clone() } }
                                    ]
                                }
                            },
                            "size": 1, // Лимит результатов
                            "_source": ["abi"], // Возвращаем только поле abi
                            "sort": [{ "block": "desc" }] // Сортируем по block в убывающем порядке
                        }))
                        .send()).unwrap();

                    // Парсим ответ
                    let response_body = rt.block_on(response.json::<Value>()).unwrap();
                    let hits = response_body["hits"]["hits"].as_array().unwrap();
                    let mut abi = &json!({});
                    for hit in hits {
                        abi = &hit["_source"]["abi"];
                    }
                    let shipper_abi = ABIEOS::new_with_abi(EOSIO_SYSTEM, abi.as_str().unwrap()).unwrap();
                    let data = shipper_abi.hex_to_json("EOSIO",a.act.account.clone().as_str(),a.act.data.as_bytes()).unwrap();
                    act = HyperionActionAct {
                        name: a.act.name.clone(),
                        account: a.act.account.clone(),
                        authorization: act_authorization,
                        data:Value::from(data),
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
                                let mut auth_sequence: Vec<AccountAuthSequence> = Vec::new();
                                for auth in &r.auth_sequence {
                                    auth_sequence.push(AccountAuthSequence{
                                        account: auth.account.clone(),
                                        sequence: auth.sequence.clone()
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
                ActionTraceVariant::action_trace_v1(a) => {}
            }

            //let new_producers = <ProducerSchedule as Clone>::clone(&header.new_producers.clone().unwrap());

            tokio::spawn(async move {
                let permit = semaphore.acquire().await.unwrap();
                let block_doc = ActionDocument {
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
                    cpu_usage_us = None;
                    net_usage_words = None;
                });
                drop(permit);
            });
        });
    }
}
