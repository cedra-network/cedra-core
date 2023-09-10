// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Support for mocking the Aptos data store.

use crate::account::AccountData;
use anyhow::Result;
use aptos_aggregator::{aggregator_extension::AggregatorID, resolver::TAggregatorResolver};
use aptos_state_view::{in_memory_state_view::InMemoryStateView, StateViewId, TStateView};
use aptos_types::{
    access_path::AccessPath,
    account_config::CoinInfoResource,
    state_store::{
        state_key::StateKey, state_storage_usage::StateStorageUsage, state_value::StateValue,
    },
    transaction::ChangeSet,
    write_set::{TransactionWrite, WriteSet},
};
use aptos_vm_genesis::{
    generate_genesis_change_set_for_mainnet, generate_genesis_change_set_for_testing,
    GenesisOptions,
};
use aptos_vm_types::resolver::{StateStorageResolver, TModuleResolver, TResourceResolver};
use move_core_types::{language_storage::ModuleId, value::MoveTypeLayout};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dummy genesis ChangeSet for testing
pub static GENESIS_CHANGE_SET_HEAD: Lazy<ChangeSet> =
    Lazy::new(|| generate_genesis_change_set_for_testing(GenesisOptions::Head));

pub static GENESIS_CHANGE_SET_TESTNET: Lazy<ChangeSet> =
    Lazy::new(|| generate_genesis_change_set_for_testing(GenesisOptions::Testnet));

pub static GENESIS_CHANGE_SET_MAINNET: Lazy<ChangeSet> =
    Lazy::new(|| generate_genesis_change_set_for_mainnet(GenesisOptions::Mainnet));

/// An in-memory implementation of `StateView` and `ExecutorResolver` for the VM.
///
/// Tests use this to set up state, and pass in a reference to the cache whenever a `StateView` or
/// `ExecutorResolver` is needed.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FakeDataStore {
    state_data: HashMap<StateKey, StateValue>,
}

impl FakeDataStore {
    /// Creates a new `FakeDataStore` with the provided initial data.
    pub fn new(data: HashMap<StateKey, Vec<u8>>) -> Self {
        FakeDataStore {
            state_data: data
                .into_iter()
                .map(|(k, v)| (k, StateValue::new_legacy(v)))
                .collect(),
        }
    }

    /// Adds a [`WriteSet`] to this data store.
    pub fn add_write_set(&mut self, write_set: &WriteSet) {
        for (state_key, write_op) in write_set {
            match write_op.as_state_value() {
                None => self.remove(state_key),
                Some(state_value) => self.set(state_key.clone(), state_value),
            };
        }
    }

    /// Sets a `(key, bytes)` pair within this data store. Wraps `bytes` in StateValue::new_legacy().
    ///
    /// Returns the previous data if the key was occupied.
    pub fn set_legacy(&mut self, state_key: StateKey, bytes: Vec<u8>) -> Option<StateValue> {
        self.state_data
            .insert(state_key, StateValue::new_legacy(bytes))
    }

    /// Sets a (key, value) pair within this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn set(&mut self, state_key: StateKey, state_value: StateValue) -> Option<StateValue> {
        self.state_data.insert(state_key, state_value)
    }

    /// Deletes a key from this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn remove(&mut self, state_key: &StateKey) -> Option<StateValue> {
        self.state_data.remove(state_key)
    }

    /// Adds an [`AccountData`] to this data store.
    pub fn add_account_data(&mut self, account_data: &AccountData) {
        let write_set = account_data.to_writeset();
        self.add_write_set(&write_set)
    }

    /// Adds CoinInfo to this data store.
    pub fn add_coin_info(&mut self) {
        let coin_info = CoinInfoResource::random(u128::MAX);
        let write_set = coin_info.to_writeset().expect("access path in test");
        self.add_write_set(&write_set)
    }

    /// Adds a `CompiledModule` to this data store.
    ///
    /// Does not do any sort of verification on the module.
    pub fn add_module(&mut self, module_id: &ModuleId, blob: Vec<u8>) {
        let access_path = AccessPath::from(module_id);
        self.set(
            StateKey::access_path(access_path),
            StateValue::new_legacy(blob),
        );
    }
}

// This is used by the `execute_block` API.
impl TStateView for FakeDataStore {
    type Key = StateKey;

    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<StateValue>> {
        Ok(self.state_data.get(state_key).cloned())
    }

    fn get_usage(&self) -> Result<StateStorageUsage> {
        let mut usage = StateStorageUsage::new_untracked();
        for (k, v) in self.state_data.iter() {
            usage.add_item(k.size() + v.size())
        }
        Ok(usage)
    }

    fn as_in_memory_state_view(&self) -> InMemoryStateView {
        InMemoryStateView::new(self.state_data.clone())
    }
}

// These traits allow `FakeDataStore` act as a resolver as well.
impl TResourceResolver for FakeDataStore {
    type Key = StateKey;
    type Layout = MoveTypeLayout;

    fn get_resource_state_value(
        &self,
        state_key: &Self::Key,
        _maybe_layout: Option<&Self::Layout>,
    ) -> Result<Option<StateValue>> {
        self.get_state_value(state_key)
    }
}

impl TModuleResolver for FakeDataStore {
    type Key = StateKey;

    fn get_module_state_value(&self, state_key: &Self::Key) -> Result<Option<StateValue>> {
        self.get_state_value(state_key)
    }
}

impl StateStorageResolver for FakeDataStore {
    fn id(&self) -> StateViewId {
        StateViewId::Miscellaneous
    }

    fn get_usage(&self) -> Result<StateStorageUsage> {
        // Re-use implementation from the `StateView`
        self.get_usage()
    }
}

impl TAggregatorResolver for FakeDataStore {
    type Key = AggregatorID;

    fn get_aggregator_v1_state_value(&self, id: &Self::Key) -> Result<Option<StateValue>> {
        self.get_state_value(id.as_state_key())
    }
}
