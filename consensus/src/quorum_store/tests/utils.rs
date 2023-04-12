// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519Signature},
    PrivateKey, Uniform,
};
use aptos_types::{
    account_address::AccountAddress,
    chain_id::ChainId,
    transaction::{RawTransaction, Script, SignedTransaction, TransactionPayload},
};

pub(crate) fn create_test_account() -> (AccountAddress, Ed25519PrivateKey) {
    (
        AccountAddress::random(),
        Ed25519PrivateKey::generate_for_testing(),
    )
}

// Creates a single test transaction for the provided account
pub(crate) fn create_signed_transaction_for_account(
    sender: &AccountAddress,
    private_key: &Ed25519PrivateKey,
    sequence_number: u64,
    gas_unit_price: u64,
) -> SignedTransaction {
    let public_key = private_key.public_key();

    let transaction_payload = TransactionPayload::Script(Script::new(vec![], vec![], vec![]));
    let raw_transaction = RawTransaction::new(
        *sender,
        sequence_number,
        transaction_payload,
        0,
        gas_unit_price,
        0,
        ChainId::new(10),
    );
    SignedTransaction::new(
        raw_transaction,
        public_key,
        Ed25519Signature::dummy_signature(),
    )
}

// Creates a single test transaction for a random account
pub(crate) fn create_signed_transaction(gas_unit_price: u64) -> SignedTransaction {
    let sender = AccountAddress::random();
    let private_key = Ed25519PrivateKey::generate_for_testing();
    create_signed_transaction_for_account(&sender, &private_key, 0, gas_unit_price)
}

pub(crate) fn create_vec_signed_transactions(size: u64) -> Vec<SignedTransaction> {
    (0..size).map(|_| create_signed_transaction(1)).collect()
}

pub(crate) fn create_vec_signed_transactions_with_gas(
    size: u64,
    gas_unit_price: u64,
) -> Vec<SignedTransaction> {
    (0..size)
        .map(|_| create_signed_transaction(gas_unit_price))
        .collect()
}
