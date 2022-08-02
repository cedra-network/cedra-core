// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::{
    current_function_name,
    protos::extractor::{
        transaction::{TransactionType, Txn_data},
        transaction_payload::{Payload, PayloadType},
        write_set_change::Change::WriteTableItem,
    },
    runtime::SfStreamer,
    tests::{new_test_context, TestContext},
};
use aptos_sdk::types::{account_config::aptos_root_address, LocalAccount};
use move_deps::move_core_types::value::MoveValue;
use move_deps::{move_core_types::account_address::AccountAddress, move_package::BuildConfig};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use std::{convert::TryInto, path::PathBuf};

#[tokio::test]
async fn test_genesis_works() {
    let test_context = new_test_context(current_function_name!());

    let context = Arc::new(test_context.context);
    let mut streamer = SfStreamer::new(context, 0, None);
    let converted = streamer.batch_convert_once(10).await;

    // position 0 should be genesis
    let txn = converted.first().unwrap().clone();
    assert_eq!(txn.version, 0);
    assert_eq!(txn.type_.unwrap(), TransactionType::GENESIS);
    assert_eq!(txn.block_height, 0);
    if let Txn_data::Genesis(txn) = txn.txn_data.unwrap() {
        assert_eq!(
            txn.events[0].key.account_address,
            aptos_root_address().to_string()
        );
    }
}

#[tokio::test]
async fn test_block_transactions_work() {
    let mut test_context = new_test_context(current_function_name!());

    // create user transactions
    let account = test_context.gen_account();
    let txn = test_context.create_user_account(&account);
    test_context.commit_block(&vec![txn.clone()]).await;

    let context = Arc::new(test_context.clone().context);
    let mut streamer = SfStreamer::new(context, 0, None);

    // emulating real stream, getting first block
    let converted_0 = streamer.batch_convert_once(1).await;
    let txn = converted_0.first().unwrap().clone();
    assert_eq!(txn.version, 0);
    assert_eq!(txn.type_.unwrap(), TransactionType::GENESIS);

    // getting second block
    let converted_1 = streamer.batch_convert_once(3).await;
    // block metadata expected
    let txn = converted_1[0].clone();
    assert_eq!(txn.version, 1);
    assert_eq!(txn.type_.unwrap(), TransactionType::BLOCK_METADATA);
    if let Txn_data::BlockMetadata(txn) = txn.txn_data.unwrap() {
        assert_eq!(txn.round, 1);
    }
    // user txn expected
    let txn = converted_1[1].clone();
    assert_eq!(txn.version, 2);
    assert_eq!(txn.type_.unwrap(), TransactionType::USER);
    if let Txn_data::User(txn) = txn.txn_data.unwrap() {
        assert_eq!(
            txn.request.payload.type_.unwrap(),
            PayloadType::SCRIPT_FUNCTION_PAYLOAD
        );
        if let Payload::ScriptFunctionPayload(payload) =
            txn.request.payload.clone().unwrap().payload.unwrap()
        {
            let address_str = MoveValue::Address(account.address()).to_string();
            let address_str = Value::String(address_str).to_string();
            assert_eq!(*payload.arguments.first().unwrap(), address_str);
        }
    }
    // state checkpoint expected
    let txn = converted_1[2].clone();
    assert_eq!(txn.version, 3);
    assert_eq!(txn.type_.unwrap(), TransactionType::STATE_CHECKPOINT);
}

#[tokio::test]
async fn test_block_height_and_ts_work() {
    let mut test_context = new_test_context(current_function_name!());

    // Creating 2 blocks w/ user transactions and 1 empty block
    let mut root_account = test_context.root_account();
    let account = test_context.gen_account();
    let txn = test_context.create_user_account_by(&mut root_account, &account);
    test_context.commit_block(&vec![txn.clone()]).await;
    let account = test_context.gen_account();
    let txn = test_context.create_user_account_by(&mut root_account, &account);
    test_context.commit_block(&vec![txn.clone()]).await;
    test_context.commit_block(&[]).await;

    // key is version and value is block_height
    let block_mapping = HashMap::from([
        (0, 0),
        (1, 1),
        (2, 1),
        (3, 1),
        (4, 2),
        (5, 2),
        (6, 2),
        (7, 3),
        (8, 3),
    ]);

    let context = Arc::new(test_context.clone().context);
    let mut streamer = SfStreamer::new(context, 0, None);

    let converted = streamer.batch_convert_once(100).await;
    let start_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Making sure that version - block height mapping is correct and that version is in order
    for (i, txn) in converted.iter().enumerate() {
        assert_eq!(txn.version as usize, i);
        assert_eq!(
            txn.block_height as usize,
            *block_mapping.get(&i).unwrap() as usize
        );
        let ts = (txn.timestamp.seconds * 1000000) as u64 + txn.timestamp.nanos as u64;
        if txn.block_height == 0 {
            // Genesis timestamp is 0
            assert_eq!(ts, 0);
        } else {
            assert_eq!(ts, start_ts + txn.block_height);
        }
    }
}

#[tokio::test]
async fn test_table_item_parsing_works() {
    let mut test_context = new_test_context(current_function_name!());
    let ctx = &mut test_context;
    let mut account = ctx.gen_account();
    let acc = &mut account;
    let txn = ctx.create_user_account(acc);
    ctx.commit_block(&vec![txn.clone()]).await;
    make_test_tables(ctx, acc).await;

    // This is a subset of k-v added from TableTestData move module
    let expected_items: HashMap<String, String> = HashMap::from([
        (json!(2).to_string(), json!(3).to_string()),
        (json!("abc").to_string(), json!("abc").to_string()),
        (json!(1).to_string(), json!(1).to_string()),
        (
            json!(["abc", "abc"]).to_string(),
            json!(["abc", "abc"]).to_string(),
        ),
    ]);

    let context = Arc::new(test_context.clone().context);
    let mut streamer = SfStreamer::new(context, 0, None);

    let converted = streamer.batch_convert_once(100).await;
    let mut table_kv: HashMap<String, String> = HashMap::new();
    for parsed_txn in converted {
        if parsed_txn.type_.unwrap() != TransactionType::USER {
            continue;
        }
        for write_set_change in parsed_txn.info.changes.clone() {
            if let WriteTableItem(item) = write_set_change.change.unwrap() {
                let data = item.data.unwrap();
                table_kv.insert(data.key, data.value);
            }
        }
    }
    for (expected_k, expected_v) in expected_items.into_iter() {
        println!(
            "Expected key: {}, expected value: {}, actual value maybe: {:?}",
            expected_k,
            expected_v,
            table_kv.get(&expected_k)
        );
        assert_eq!(table_kv.get(&expected_k).unwrap(), &expected_v);
    }
}

async fn make_test_tables(ctx: &mut TestContext, account: &mut LocalAccount) {
    let module = build_test_module(account.address()).await;

    ctx.api_publish_module(account, module.try_into().unwrap())
        .await;
    ctx.api_execute_script_function(
        account,
        "TableTestData",
        "make_test_tables",
        json!([]),
        json!([]),
    )
    .await
}

async fn build_test_module(account: AccountAddress) -> Vec<u8> {
    let package_dir = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("api/move-test-package");
    let build_config = BuildConfig {
        generate_docs: false,
        install_dir: Some(package_dir.clone()),
        additional_named_addresses: [("TestAccount".to_string(), account)].into(),
        ..Default::default()
    };
    let package = build_config
        .compile_package(&package_dir, &mut Vec::new())
        .unwrap();

    let mut out = Vec::new();
    package
        .root_modules_map()
        .iter_modules()
        .first()
        .unwrap()
        .serialize(&mut out)
        .unwrap();
    out
}
