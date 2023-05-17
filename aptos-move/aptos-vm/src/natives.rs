// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use aptos_aggregator::resolver::AggregatorResolver;
#[cfg(feature = "testing")]
use aptos_framework::natives::cryptography::algebra::AlgebraContext;
use aptos_gas::{AbstractValueSizeGasParameters, NativeGasParameters, LATEST_GAS_FEATURE_VERSION};
#[cfg(feature = "testing")]
use aptos_types::chain_id::ChainId;
use aptos_types::{
    account_config::CORE_CODE_ADDRESS,
    on_chain_config::{Features, TimedFeatures},
    state_store::table::TableHandle,
};
use move_vm_runtime::native_functions::NativeFunctionTable;
use std::{ops::Deref, sync::Arc};
#[cfg(feature = "testing")]
use {
    aptos_framework::natives::{
        aggregator_natives::NativeAggregatorContext, code::NativeCodeContext,
        cryptography::ristretto255_point::NativeRistrettoPointContext,
        transaction_context::NativeTransactionContext,
    },
    move_vm_runtime::native_extensions::NativeContextExtensions,
    move_vm_test_utils::BlankStorage,
    once_cell::sync::Lazy,
};

// We need this adapter so that we can implement Aptos resolver traits.
struct BlankStorageAdapter(BlankStorage);

impl BlankStorageAdapter {
    pub fn new() -> Self {
        Self(BlankStorage)
    }
}

// Implement Deref so that all Move resolver traits still apply.
impl Deref for BlankStorageAdapter {
    type Target = BlankStorage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Needed for unit testing aggregator.
impl AggregatorResolver for BlankStorageAdapter {
    fn resolve_aggregator_value(
        &self,
        _handle: &TableHandle,
        _key: &[u8],
    ) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(None)
    }
}

#[cfg(feature = "testing")]
static DUMMY_RESOLVER: Lazy<BlankStorageAdapter> = Lazy::new(BlankStorageAdapter::new);

pub fn aptos_natives(
    gas_params: NativeGasParameters,
    abs_val_size_gas_params: AbstractValueSizeGasParameters,
    gas_feature_version: u64,
    timed_features: TimedFeatures,
    features: Arc<Features>,
) -> NativeFunctionTable {
    aptos_move_stdlib::natives::all_natives(CORE_CODE_ADDRESS, gas_params.move_stdlib.clone())
        .into_iter()
        .filter(|(_, name, _, _)| name.as_str() != "vector")
        .chain(aptos_framework::natives::all_natives(
            CORE_CODE_ADDRESS,
            gas_params.move_stdlib,
            gas_params.aptos_framework,
            timed_features,
            features,
            move |val| abs_val_size_gas_params.abstract_value_size(val, gas_feature_version),
        ))
        .chain(move_table_extension::table_natives(
            CORE_CODE_ADDRESS,
            gas_params.table,
        ))
        .collect()
}

pub fn assert_no_test_natives(err_msg: &str) {
    assert!(
        aptos_natives(
            NativeGasParameters::zeros(),
            AbstractValueSizeGasParameters::zeros(),
            LATEST_GAS_FEATURE_VERSION,
            TimedFeatures::enable_all(),
            Arc::new(Features::default())
        )
        .into_iter()
        .all(|(_, module_name, func_name, _)| {
            !(module_name.as_str() == "unit_test"
                && func_name.as_str() == "create_signers_for_testing"
                || module_name.as_str() == "ed25519"
                    && func_name.as_str() == "generate_keys_internal"
                || module_name.as_str() == "ed25519" && func_name.as_str() == "sign_internal"
                || module_name.as_str() == "multi_ed25519"
                    && func_name.as_str() == "generate_keys_internal"
                || module_name.as_str() == "multi_ed25519" && func_name.as_str() == "sign_internal"
                || module_name.as_str() == "bls12381"
                    && func_name.as_str() == "generate_keys_internal"
                || module_name.as_str() == "bls12381" && func_name.as_str() == "sign_internal"
                || module_name.as_str() == "bls12381"
                    && func_name.as_str() == "generate_proof_of_possession_internal")
        }),
        "{}",
        err_msg
    )
}

#[cfg(feature = "testing")]
pub fn configure_for_unit_test() {
    move_unit_test::extensions::set_extension_hook(Box::new(unit_test_extensions_hook))
}

#[cfg(feature = "testing")]
fn unit_test_extensions_hook(exts: &mut NativeContextExtensions) {
    exts.add(NativeCodeContext::default());
    exts.add(NativeTransactionContext::new(vec![1], ChainId::test().id())); // We use the testing environment chain ID here
    exts.add(NativeAggregatorContext::new([0; 32], &*DUMMY_RESOLVER));
    exts.add(NativeRistrettoPointContext::new());
    exts.add(AlgebraContext::new());
}
