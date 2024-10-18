// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    captured_reads::CacheRead,
    cross_block_caches::CrossBlockModuleCache,
    view::{LatestView, ViewState},
};
use ambassador::delegate_to_methods;
use aptos_mvhashmap::types::TxnIndex;
use aptos_types::{
    executable::{Executable, ModulePath},
    state_store::{state_value::StateValueMetadata, TStateView},
    transaction::BlockExecutableTransaction as Transaction,
    vm::modules::AptosModuleExtension,
};
use aptos_vm_types::module_and_script_storage::module_storage::AptosModuleStorage;
use move_binary_format::{
    errors::{Location, PartialVMResult, VMResult},
    file_format::CompiledScript,
    CompiledModule,
};
use move_core_types::{
    account_address::AccountAddress, identifier::IdentStr, language_storage::ModuleId,
};
use move_vm_runtime::{Module, RuntimeEnvironment, Script, WithRuntimeEnvironment};
use move_vm_types::code::{
    ambassador_impl_ScriptCache, Code, ModuleCache, ModuleCode, ModuleCodeBuilder, ScriptCache,
    WithBytes,
};
use std::sync::Arc;

impl<'a, T: Transaction, S: TStateView<Key = T::Key>, X: Executable> WithRuntimeEnvironment
    for LatestView<'a, T, S, X>
{
    fn runtime_environment(&self) -> &RuntimeEnvironment {
        self.runtime_environment
    }
}

impl<'a, T: Transaction, S: TStateView<Key = T::Key>, X: Executable> ModuleCodeBuilder
    for LatestView<'a, T, S, X>
{
    type Deserialized = CompiledModule;
    type Extension = AptosModuleExtension;
    type Key = ModuleId;
    type Verified = Module;
    type Version = Option<TxnIndex>;

    fn build(
        &self,
        key: &Self::Key,
    ) -> VMResult<
        Option<ModuleCode<Self::Deserialized, Self::Verified, Self::Extension, Self::Version>>,
    > {
        let key = T::Key::from_address_and_module_name(key.address(), key.name());
        self.get_raw_base_value(&key)
            .map_err(|err| err.finish(Location::Undefined))?
            .map(|state_value| {
                let extension = AptosModuleExtension::new(state_value);
                let (compiled_module, _, _) = self
                    .runtime_environment()
                    .deserialize_into_compiled_module(extension.bytes())?;
                let version = None;
                Ok(ModuleCode::from_deserialized(
                    compiled_module,
                    Arc::new(extension),
                    version,
                ))
            })
            .transpose()
    }
}

impl<'a, T: Transaction, S: TStateView<Key = T::Key>, X: Executable> ModuleCache
    for LatestView<'a, T, S, X>
{
    type Deserialized = CompiledModule;
    type Extension = AptosModuleExtension;
    type Key = ModuleId;
    type Verified = Module;
    type Version = Option<TxnIndex>;

    fn insert_deserialized_module(
        &self,
        key: Self::Key,
        deserialized_code: Self::Deserialized,
        extension: Arc<Self::Extension>,
        version: Self::Version,
    ) -> VMResult<()> {
        self.as_module_cache().insert_deserialized_module(
            key,
            deserialized_code,
            extension,
            version,
        )
    }

    fn insert_verified_module(
        &self,
        key: Self::Key,
        verified_code: Self::Verified,
        extension: Arc<Self::Extension>,
        version: Self::Version,
    ) -> VMResult<Arc<ModuleCode<Self::Deserialized, Self::Verified, Self::Extension, Self::Version>>>
    {
        match &self.latest_view {
            ViewState::Sync(state) => {
                // For parallel execution, if we insert a verified module, we might need to also
                // update module cache in captured reads so that they also store the verified code.
                // If we do not do that, reads to module cache will end up reading "old" code that
                // is stored in captured reads and is not verified.
                let module = state.versioned_map.module_cache().insert_verified_module(
                    key.clone(),
                    verified_code,
                    extension,
                    version,
                )?;
                state
                    .captured_reads
                    .borrow_mut()
                    .capture_per_block_cache_read(key, Some(module.clone()));
                Ok(module)
            },
            ViewState::Unsync(state) => state.unsync_map.module_cache().insert_verified_module(
                key,
                verified_code,
                extension,
                version,
            ),
        }
    }

    fn get_module_or_build_with(
        &self,
        key: &Self::Key,
        builder: &dyn ModuleCodeBuilder<
            Key = Self::Key,
            Deserialized = Self::Deserialized,
            Verified = Self::Verified,
            Extension = Self::Extension,
            Version = Self::Version,
        >,
    ) -> VMResult<
        Option<Arc<ModuleCode<Self::Deserialized, Self::Verified, Self::Extension, Self::Version>>>,
    > {
        // First, look up the module in the cross-block global module cache. Record the read for
        // later validation in case the read module is republished.
        if let Some(module) = CrossBlockModuleCache::get(key) {
            match &self.latest_view {
                ViewState::Sync(state) => state
                    .captured_reads
                    .borrow_mut()
                    .capture_global_cache_read(key.clone()),
                ViewState::Unsync(state) => {
                    state.read_set.borrow_mut().capture_module_read(key.clone())
                },
            }
            return Ok(Some(module.clone()));
        }

        // Global cache miss: check module cache in versioned/unsync maps.
        match &self.latest_view {
            ViewState::Sync(state) => {
                // Check the transaction-level cache with already read modules first.
                let cache_read = state.captured_reads.borrow().get_module_read(key)?;
                match cache_read {
                    CacheRead::Hit(read) => Ok(read),
                    CacheRead::Miss => {
                        // If the module has not been accessed by this transaction, go to the
                        // module cache and record the read.
                        let read = state
                            .versioned_map
                            .module_cache()
                            .get_module_or_build_with(key, builder)?;
                        state
                            .captured_reads
                            .borrow_mut()
                            .capture_per_block_cache_read(key.clone(), read.clone());
                        Ok(read)
                    },
                }
            },
            ViewState::Unsync(state) => {
                let read = state
                    .unsync_map
                    .module_cache()
                    .get_module_or_build_with(key, builder)?;
                state.read_set.borrow_mut().capture_module_read(key.clone());
                Ok(read)
            },
        }
    }

    fn num_modules(&self) -> usize {
        self.as_module_cache().num_modules()
    }
}

impl<'a, T: Transaction, S: TStateView<Key = T::Key>, X: Executable> AptosModuleStorage
    for LatestView<'a, T, S, X>
{
    fn fetch_state_value_metadata(
        &self,
        address: &AccountAddress,
        module_name: &IdentStr,
    ) -> PartialVMResult<Option<StateValueMetadata>> {
        let id = ModuleId::new(*address, module_name.to_owned());
        let state_value_metadata = self
            .get_module_or_build_with(&id, self)
            .map_err(|err| err.to_partial())?
            .map(|module| module.extension().state_value_metadata().clone());
        Ok(state_value_metadata)
    }
}

#[delegate_to_methods]
#[delegate(ScriptCache, target_ref = "as_script_cache")]
impl<'a, T: Transaction, S: TStateView<Key = T::Key>, X: Executable> LatestView<'a, T, S, X> {
    /// Returns the script cache.
    fn as_script_cache(
        &self,
    ) -> &dyn ScriptCache<Key = [u8; 32], Deserialized = CompiledScript, Verified = Script> {
        match &self.latest_view {
            ViewState::Sync(state) => state.versioned_map.script_cache(),
            ViewState::Unsync(state) => state.unsync_map.script_cache(),
        }
    }

    /// Returns the module cache.
    fn as_module_cache(
        &self,
    ) -> &dyn ModuleCache<
        Key = ModuleId,
        Deserialized = CompiledModule,
        Verified = Module,
        Extension = AptosModuleExtension,
        Version = Option<TxnIndex>,
    > {
        match &self.latest_view {
            ViewState::Sync(state) => state.versioned_map.module_cache(),
            ViewState::Unsync(state) => state.unsync_map.module_cache(),
        }
    }
}
