// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::smoke_test_environment::SwarmBuilder;
use aptos::account::create::DEFAULT_FUNDED_COINS;
use aptos::common::types::{GasOptions, DEFAULT_GAS_UNIT_PRICE, DEFAULT_MAX_GAS};
use aptos_crypto::PrivateKey;
use aptos_keygen::KeyGen;
use aptos_types::{
    account_address::AccountAddress, account_config::CORE_CODE_ADDRESS,
    transaction::authenticator::AuthenticationKey,
};
use forge::NodeExt;

#[tokio::test]
async fn test_account_flow() {
    let (_swarm, mut cli, _faucet) = SwarmBuilder::new_local(1)
        .with_aptos()
        .build_with_cli(2)
        .await;

    cli.assert_account_balance_now(0, DEFAULT_FUNDED_COINS)
        .await;
    cli.assert_account_balance_now(1, DEFAULT_FUNDED_COINS)
        .await;

    let transfer_amount = 100;
    let response = cli
        .transfer_coins(
            0,
            1,
            transfer_amount,
            Some(GasOptions {
                gas_unit_price: DEFAULT_GAS_UNIT_PRICE * 2,
                max_gas: DEFAULT_MAX_GAS,
            }),
        )
        .await
        .unwrap();
    let expected_sender_amount =
        DEFAULT_FUNDED_COINS - (response.gas_used * response.gas_unit_price) - transfer_amount;
    let expected_receiver_amount = DEFAULT_FUNDED_COINS + transfer_amount;

    // transfer_coins already waits for transaction to be committed
    cli.assert_account_balance_now(0, expected_sender_amount)
        .await;
    cli.assert_account_balance_now(1, expected_receiver_amount)
        .await;

    let expected_sender_amount = expected_sender_amount + DEFAULT_FUNDED_COINS;
    let _ = cli.fund_account(0, None).await.unwrap();
    // fund_account already waits for transaction to be committed
    cli.assert_account_balance_now(0, expected_sender_amount)
        .await;

    // Create another cli account:
    cli.create_cli_account_from_faucet(KeyGen::from_os_rng().generate_ed25519_private_key(), None)
        .await
        .unwrap();
    cli.assert_account_balance_now(2, DEFAULT_FUNDED_COINS)
        .await;
}

#[tokio::test]
async fn test_account_key_rotation() {
    let (swarm, cli, _faucet) = SwarmBuilder::new_local(1)
        .with_aptos()
        .build_with_cli(1)
        .await;

    let mut keygen = KeyGen::from_os_rng();
    let new_private_key = keygen.generate_ed25519_private_key();

    cli.rotate_key(0, hex::encode(new_private_key.to_bytes()))
        .await
        .unwrap();

    let rest_client = swarm.validators().next().unwrap().rest_client();

    let originating_resource = rest_client
        .get_account_resource(CORE_CODE_ADDRESS, "0x1::account::OriginatingAddress")
        .await
        .unwrap()
        .into_inner()
        .unwrap()
        .data;

    let table_handle = originating_resource["address_map"]["handle"]
        .as_str()
        .unwrap();

    let new_address = AuthenticationKey::ed25519(&new_private_key.public_key()).derived_address();

    assert_eq!(
        AccountAddress::from_hex_literal(
            rest_client
                .get_table_item(
                    table_handle,
                    "address",
                    "address",
                    new_address.to_hex_literal(),
                )
                .await
                .unwrap()
                .into_inner()
                .as_str()
                .unwrap()
        )
        .unwrap(),
        cli.account_id(0)
    );
}
