// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

//! This file defines state store APIs that are related account state Merkle tree.

// FIXME(aldenhu)
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::{
    ledger_db::LedgerDb,
    metrics::{OTHER_TIMERS_SECONDS, STATE_ITEMS, TOTAL_STATE_BYTES},
    pruner::{StateKvPrunerManager, StateMerklePrunerManager},
    schema::{
        db_metadata::{DbMetadataKey, DbMetadataSchema, DbMetadataValue},
        stale_node_index::StaleNodeIndexSchema,
        stale_node_index_cross_epoch::StaleNodeIndexCrossEpochSchema,
        stale_state_value_index::StaleStateValueIndexSchema,
        stale_state_value_index_by_key_hash::StaleStateValueIndexByKeyHashSchema,
        state_value::StateValueSchema,
        state_value_by_key_hash::StateValueByKeyHashSchema,
        version_data::VersionDataSchema,
    },
    state_kv_db::StateKvDb,
    state_merkle_db::StateMerkleDb,
    state_restore::{StateSnapshotRestore, StateSnapshotRestoreMode, StateValueWriter},
    state_store::{
        buffered_state::BufferedState, current_state::CurrentState, persisted_state::PersistedState,
    },
    utils::{
        iterators::PrefixedStateValueIterator,
        new_sharded_kv_schema_batch,
        truncation_helper::{
            find_tree_root_at_or_before, get_max_version_in_state_merkle_db, truncate_ledger_db,
            truncate_state_kv_db, truncate_state_merkle_db,
        },
        ShardedStateKvSchemaBatch,
    },
};
use aptos_crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use aptos_db_indexer::db_indexer::InternalIndexerDB;
use aptos_db_indexer_schemas::{
    metadata::{MetadataKey, MetadataValue, StateSnapshotProgress},
    schema::indexer_metadata::InternalIndexerMetadataSchema,
};
use aptos_executor::types::in_memory_state_calculator_v2::InMemoryStateCalculatorV2;
use aptos_experimental_runtimes::thread_manager::THREAD_MANAGER;
use aptos_infallible::Mutex;
use aptos_jellyfish_merkle::iterator::JellyfishMerkleIterator;
use aptos_logger::info;
use aptos_metrics_core::TimerHelper;
use aptos_schemadb::SchemaBatch;
use aptos_scratchpad::SparseMerkleTree;
use aptos_storage_interface::{
    db_ensure as ensure, db_other_bail as bail,
    state_store::{
        sharded_state_update_refs::{ShardedStateUpdateRefs, StateUpdateRefWithOffset},
        state::State,
        state_delta::StateDelta,
        state_update::StateValueWithVersionOpt,
        state_view::{
            async_proof_fetcher::AsyncProofFetcher,
            cached_state_view::{CachedStateView, ShardedStateCache, StateCacheShard},
        },
        NUM_STATE_SHARDS,
    },
    AptosDbError, DbReader, Result, StateSnapshotReceiver,
};
use aptos_types::{
    proof::{definition::LeafCount, SparseMerkleProofExt, SparseMerkleRangeProof},
    state_store::{
        state_key::{prefix::StateKeyPrefix, StateKey},
        state_storage_usage::StateStorageUsage,
        state_value::{
            StaleStateValueByKeyHashIndex, StaleStateValueIndex, StateValue,
            StateValueChunkWithProof,
        },
        StateViewId,
    },
    transaction::Version,
    write_set::WriteSet,
};
use arc_swap::ArcSwap;
use claims::{assert_ge, assert_le};
use itertools::Itertools;
use rayon::prelude::*;
use std::{
    collections::HashSet,
    ops::Deref,
    sync::{Arc, MutexGuard},
};

pub(crate) mod buffered_state;
mod state_merkle_batch_committer;
mod state_snapshot_committer;

mod current_state;
mod persisted_state;
#[cfg(test)]
mod state_store_test;

type StateValueBatch = crate::state_restore::StateValueBatch<StateKey, Option<StateValue>>;

// We assume TARGET_SNAPSHOT_INTERVAL_IN_VERSION > block size.
const MAX_WRITE_SETS_AFTER_SNAPSHOT: LeafCount = buffered_state::TARGET_SNAPSHOT_INTERVAL_IN_VERSION
    * (buffered_state::ASYNC_COMMIT_CHANNEL_BUFFER_SIZE + 2 + 1/*  Rendezvous channel */)
    * 2;

pub const MAX_COMMIT_PROGRESS_DIFFERENCE: u64 = 1_000_000;

pub(crate) struct StateDb {
    pub ledger_db: Arc<LedgerDb>,
    pub state_merkle_db: Arc<StateMerkleDb>,
    pub state_kv_db: Arc<StateKvDb>,
    pub state_merkle_pruner: StateMerklePrunerManager<StaleNodeIndexSchema>,
    pub epoch_snapshot_pruner: StateMerklePrunerManager<StaleNodeIndexCrossEpochSchema>,
    pub state_kv_pruner: StateKvPrunerManager,
    pub skip_usage: bool,
}

pub(crate) struct StateStore {
    pub state_db: Arc<StateDb>,
    /// The `base` of buffered_state is the latest snapshot in state_merkle_db while `current`
    /// is the latest state sparse merkle tree that is replayed from that snapshot until the latest
    /// write set stored in ledger_db.
    buffered_state: Mutex<BufferedState>,
    /// CurrentState is shared between this and the buffered_state.
    /// On read, we don't need to lock the `buffered_state` to get the latest state.
    current_state: Arc<Mutex<CurrentState>>,
    /// Tracks a persisted smt, any state older than that is guaranteed to be found in RocksDB
    persisted_state: Arc<Mutex<PersistedState>>,
    buffered_state_target_items: usize,
    internal_indexer_db: Option<InternalIndexerDB>,
}

impl Deref for StateStore {
    type Target = StateDb;

    fn deref(&self) -> &Self::Target {
        self.state_db.deref()
    }
}

// "using an Arc<dyn DbReader> as an Arc<dyn StateReader>" is not allowed in stable Rust. Actually we
// want another trait, `StateReader`, which is a subset of `DbReader` here but Rust does not support trait
// upcasting coercion for now. Should change it to a different trait once upcasting is stabilized.
// ref: https://github.com/rust-lang/rust/issues/65991
impl DbReader for StateDb {
    /// Returns the latest state snapshot strictly before `next_version` if any.
    fn get_state_snapshot_before(
        &self,
        next_version: Version,
    ) -> Result<Option<(Version, HashValue)>> {
        self.state_merkle_db
            .get_state_snapshot_version_before(next_version)?
            .map(|ver| Ok((ver, self.state_merkle_db.get_root_hash(ver)?)))
            .transpose()
    }

    /// Get the latest state value of the given key up to the given version. Only used for testing for now
    /// but should replace the `get_value_with_proof_by_version` call for VM execution if just fetch the
    /// value without proof.
    fn get_state_value_by_version(
        &self,
        state_key: &StateKey,
        version: Version,
    ) -> Result<Option<StateValue>> {
        Ok(self
            .get_state_value_with_version_by_version(state_key, version)?
            .map(|(_, value)| value))
    }

    /// Gets the latest state value and its corresponding version when it's of the given key up
    /// to the given version.
    fn get_state_value_with_version_by_version(
        &self,
        state_key: &StateKey,
        version: Version,
    ) -> Result<Option<(Version, StateValue)>> {
        self.state_kv_db
            .get_state_value_with_version_by_version(state_key, version)
    }

    /// Returns the proof of the given state key and version.
    fn get_state_proof_by_version_ext(
        &self,
        state_key: &StateKey,
        version: Version,
        root_depth: usize,
    ) -> Result<SparseMerkleProofExt> {
        let (_, proof) = self
            .state_merkle_db
            .get_with_proof_ext(state_key, version, root_depth)?;
        Ok(proof)
    }

    /// Get the state value with proof given the state key and version
    fn get_state_value_with_proof_by_version_ext(
        &self,
        state_key: &StateKey,
        version: Version,
        root_depth: usize,
    ) -> Result<(Option<StateValue>, SparseMerkleProofExt)> {
        let (leaf_data, proof) = self
            .state_merkle_db
            .get_with_proof_ext(state_key, version, root_depth)?;
        Ok((
            match leaf_data {
                Some((_, (key, version))) => Some(self.expect_value_by_version(&key, version)?),
                None => None,
            },
            proof,
        ))
    }

    fn get_state_storage_usage(&self, version: Option<Version>) -> Result<StateStorageUsage> {
        version.map_or(Ok(StateStorageUsage::zero()), |version| {
            Ok(match self.ledger_db.metadata_db().get_usage(version) {
                Ok(data) => data,
                _ => {
                    ensure!(self.skip_usage, "VersionData at {version} is missing.");
                    StateStorageUsage::new_untracked()
                },
            })
        })
    }
}

impl DbReader for StateStore {
    fn get_buffered_state_base(&self) -> Result<SparseMerkleTree<StateValue>> {
        Ok(self.persisted_state().clone())
    }

    /// Returns the latest state snapshot strictly before `next_version` if any.
    fn get_state_snapshot_before(
        &self,
        next_version: Version,
    ) -> Result<Option<(Version, HashValue)>> {
        self.deref().get_state_snapshot_before(next_version)
    }

    /// Get the latest state value of the given key up to the given version. Only used for testing for now
    /// but should replace the `get_value_with_proof_by_version` call for VM execution if just fetch the
    /// value without proof.
    fn get_state_value_by_version(
        &self,
        state_key: &StateKey,
        version: Version,
    ) -> Result<Option<StateValue>> {
        self.deref().get_state_value_by_version(state_key, version)
    }

    /// Gets the latest state value and the its corresponding version when its of the given key up
    /// to the given version.
    fn get_state_value_with_version_by_version(
        &self,
        state_key: &StateKey,
        version: Version,
    ) -> Result<Option<(Version, StateValue)>> {
        self.deref()
            .get_state_value_with_version_by_version(state_key, version)
    }

    /// Returns the proof of the given state key and version.
    fn get_state_proof_by_version_ext(
        &self,
        state_key: &StateKey,
        version: Version,
        root_depth: usize,
    ) -> Result<SparseMerkleProofExt> {
        self.deref()
            .get_state_proof_by_version_ext(state_key, version, root_depth)
    }

    /// Get the state value with proof extension given the state key and version
    fn get_state_value_with_proof_by_version_ext(
        &self,
        state_key: &StateKey,
        version: Version,
        root_depth: usize,
    ) -> Result<(Option<StateValue>, SparseMerkleProofExt)> {
        self.deref()
            .get_state_value_with_proof_by_version_ext(state_key, version, root_depth)
    }
}

impl StateDb {
    fn expect_value_by_version(
        &self,
        state_key: &StateKey,
        version: Version,
    ) -> Result<StateValue> {
        self.get_state_value_by_version(state_key, version)
            .and_then(|opt| {
                opt.ok_or_else(|| {
                    AptosDbError::NotFound(format!(
                        "State Value is missing for key {:?} by version {}",
                        state_key, version
                    ))
                })
            })
    }
}

impl StateStore {
    pub fn new(
        ledger_db: Arc<LedgerDb>,
        state_merkle_db: Arc<StateMerkleDb>,
        state_kv_db: Arc<StateKvDb>,
        state_merkle_pruner: StateMerklePrunerManager<StaleNodeIndexSchema>,
        epoch_snapshot_pruner: StateMerklePrunerManager<StaleNodeIndexCrossEpochSchema>,
        state_kv_pruner: StateKvPrunerManager,
        buffered_state_target_items: usize,
        hack_for_tests: bool,
        empty_buffered_state_for_restore: bool,
        skip_usage: bool,
        internal_indexer_db: Option<InternalIndexerDB>,
    ) -> Self {
        if !hack_for_tests && !empty_buffered_state_for_restore {
            Self::sync_commit_progress(
                Arc::clone(&ledger_db),
                Arc::clone(&state_kv_db),
                Arc::clone(&state_merkle_db),
                /*crash_if_difference_is_too_large=*/ true,
            );
        }
        let state_db = Arc::new(StateDb {
            ledger_db,
            state_merkle_db,
            state_kv_db,
            state_merkle_pruner,
            epoch_snapshot_pruner,
            state_kv_pruner,
            skip_usage,
        });
        let current_state = Arc::new(Mutex::new(CurrentState::new_dummy()));
        let persisted_state = Arc::new(Mutex::new(PersistedState::new_dummy()));
        let buffered_state = if empty_buffered_state_for_restore {
            BufferedState::new(
                &state_db,
                StateDelta::new_empty(),
                buffered_state_target_items,
                current_state.clone(),
                persisted_state.clone(),
            )
        } else {
            Self::create_buffered_state_from_latest_snapshot(
                &state_db,
                buffered_state_target_items,
                hack_for_tests,
                /*check_max_versions_after_snapshot=*/ true,
                current_state.clone(),
                persisted_state.clone(),
            )
            .expect("buffered state creation failed.")
        };

        Self {
            state_db,
            buffered_state: Mutex::new(buffered_state),
            buffered_state_target_items,
            current_state,
            persisted_state,
            internal_indexer_db,
        }
    }

    // We commit the overall commit progress at the last, and use it as the source of truth of the
    // commit progress.
    pub fn sync_commit_progress(
        ledger_db: Arc<LedgerDb>,
        state_kv_db: Arc<StateKvDb>,
        state_merkle_db: Arc<StateMerkleDb>,
        crash_if_difference_is_too_large: bool,
    ) {
        let ledger_metadata_db = ledger_db.metadata_db();
        if let Some(overall_commit_progress) = ledger_metadata_db
            .get_synced_version()
            .expect("DB read failed.")
        {
            info!(
                overall_commit_progress = overall_commit_progress,
                "Start syncing databases..."
            );
            let ledger_commit_progress = ledger_metadata_db
                .get_ledger_commit_progress()
                .expect("Failed to read ledger commit progress.");
            assert_ge!(ledger_commit_progress, overall_commit_progress);

            let state_kv_commit_progress = state_kv_db
                .metadata_db()
                .get::<DbMetadataSchema>(&DbMetadataKey::StateKvCommitProgress)
                .expect("Failed to read state K/V commit progress.")
                .expect("State K/V commit progress cannot be None.")
                .expect_version();
            assert_ge!(state_kv_commit_progress, overall_commit_progress);

            // LedgerCommitProgress was not guaranteed to commit after all ledger changes finish,
            // have to attempt truncating every column family.
            info!(
                ledger_commit_progress = ledger_commit_progress,
                "Attempt ledger truncation...",
            );
            let difference = ledger_commit_progress - overall_commit_progress;
            if crash_if_difference_is_too_large {
                assert_le!(difference, MAX_COMMIT_PROGRESS_DIFFERENCE);
            }
            truncate_ledger_db(ledger_db, overall_commit_progress)
                .expect("Failed to truncate ledger db.");

            // State K/V commit progress isn't (can't be) written atomically with the data,
            // because there are shards, so we have to attempt truncation anyway.
            info!(
                state_kv_commit_progress = state_kv_commit_progress,
                "Start state KV truncation..."
            );
            let difference = state_kv_commit_progress - overall_commit_progress;
            if crash_if_difference_is_too_large {
                assert_le!(difference, MAX_COMMIT_PROGRESS_DIFFERENCE);
            }
            truncate_state_kv_db(
                &state_kv_db,
                state_kv_commit_progress,
                overall_commit_progress,
                std::cmp::max(difference as usize, 1), /* batch_size */
            )
            .expect("Failed to truncate state K/V db.");

            let state_merkle_max_version = get_max_version_in_state_merkle_db(&state_merkle_db)
                .expect("Failed to get state merkle max version.")
                .expect("State merkle max version cannot be None.");
            if state_merkle_max_version > overall_commit_progress {
                let difference = state_merkle_max_version - overall_commit_progress;
                if crash_if_difference_is_too_large {
                    assert_le!(difference, MAX_COMMIT_PROGRESS_DIFFERENCE);
                }
            }
            let db = state_merkle_db.metadata_db();
            let state_merkle_target_version =
                find_tree_root_at_or_before(db, &state_merkle_db, overall_commit_progress)
                    .expect("DB read failed.")
                    .unwrap_or_else(|| {
                        panic!(
                    "Could not find a valid root before or at version {}, maybe it was pruned?",
                    overall_commit_progress
                )
                    });
            if state_merkle_target_version < state_merkle_max_version {
                info!(
                    state_merkle_max_version = state_merkle_max_version,
                    target_version = state_merkle_target_version,
                    "Start state merkle truncation..."
                );
                truncate_state_merkle_db(&state_merkle_db, state_merkle_target_version)
                    .expect("Failed to truncate state merkle db.");
            }
        } else {
            info!("No overall commit progress was found!");
        }
    }

    #[cfg(feature = "db-debugger")]
    pub fn catch_up_state_merkle_db(
        ledger_db: Arc<LedgerDb>,
        state_merkle_db: Arc<StateMerkleDb>,
        state_kv_db: Arc<StateKvDb>,
    ) -> Result<Option<Version>> {
        /*
        use aptos_config::config::NO_OP_STORAGE_PRUNER_CONFIG;

        let state_merkle_pruner = StateMerklePrunerManager::new(
            Arc::clone(&state_merkle_db),
            NO_OP_STORAGE_PRUNER_CONFIG.state_merkle_pruner_config,
        );
        let epoch_snapshot_pruner = StateMerklePrunerManager::new(
            Arc::clone(&state_merkle_db),
            NO_OP_STORAGE_PRUNER_CONFIG.state_merkle_pruner_config,
        );
        let state_kv_pruner = StateKvPrunerManager::new(
            Arc::clone(&state_kv_db),
            NO_OP_STORAGE_PRUNER_CONFIG.ledger_pruner_config,
        );
        let state_db = Arc::new(StateDb {
            ledger_db,
            state_merkle_db,
            state_kv_db,
            state_merkle_pruner,
            epoch_snapshot_pruner,
            state_kv_pruner,
            skip_usage: false,
        });
        let current_state = Arc::new(Mutex::new(CurrentState::new_dummy()));
        let persisted_state = Arc::new(Mutex::new(PersistedState::new_dummy()));
        let _ = Self::create_buffered_state_from_latest_snapshot(
            &state_db,
            0,
            /*hack_for_tests=*/ false,
            /*check_max_versions_after_snapshot=*/ false,
            current_state.clone(),
            persisted_state,
        )?;
        let base_version = current_state.lock().base_version;
        Ok(base_version)
        FIXME(aldenhu)
         */
        todo!()
    }

    fn create_buffered_state_from_latest_snapshot(
        state_db: &Arc<StateDb>,
        buffered_state_target_items: usize,
        hack_for_tests: bool,
        check_max_versions_after_snapshot: bool,
        current_state: Arc<Mutex<CurrentState>>,
        persisted_state: Arc<Mutex<PersistedState>>,
    ) -> Result<BufferedState> {
        /*
        let num_transactions = state_db
            .ledger_db
            .metadata_db()
            .get_synced_version()?
            .map_or(0, |v| v + 1);

        let latest_snapshot_version = state_db
            .state_merkle_db
            .get_state_snapshot_version_before(Version::MAX)
            .expect("Failed to query latest node on initialization.");

        info!(
            num_transactions = num_transactions,
            latest_snapshot_version = latest_snapshot_version,
            "Initializing BufferedState."
        );
        let latest_snapshot_root_hash = if let Some(version) = latest_snapshot_version {
            state_db
                .state_merkle_db
                .get_root_hash(version)
                .expect("Failed to query latest checkpoint root hash on initialization.")
        } else {
            *SPARSE_MERKLE_PLACEHOLDER_HASH
        };
        let usage = state_db.get_state_storage_usage(latest_snapshot_version)?;
        let mut buffered_state = BufferedState::new(
            state_db,
            StateDelta::new_at_checkpoint(
                latest_snapshot_root_hash,
                usage,
                latest_snapshot_version,
            ),
            buffered_state_target_items,
            current_state.clone(),
            persisted_state,
        );

        // In some backup-restore tests we hope to open the db without consistency check.
        if hack_for_tests {
            return Ok(buffered_state);
        }

        // Make sure the committed transactions is ahead of the latest snapshot.
        let snapshot_next_version = latest_snapshot_version.map_or(0, |v| v + 1);

        // For non-restore cases, always snapshot_next_version <= num_transactions.
        if snapshot_next_version > num_transactions {
            info!(
                snapshot_next_version = snapshot_next_version,
                num_transactions = num_transactions,
                "snapshot is after latest transaction version. It should only happen in restore mode",
            );
        }

        // Replaying the committed write sets after the latest snapshot.
        if snapshot_next_version < num_transactions {
            if check_max_versions_after_snapshot {
                ensure!(
                    num_transactions - snapshot_next_version <= MAX_WRITE_SETS_AFTER_SNAPSHOT,
                    "Too many versions after state snapshot. snapshot_next_version: {}, num_transactions: {}",
                    snapshot_next_version,
                    num_transactions,
                );
            }
            let snapshot = state_db.get_state_snapshot_before(num_transactions)?;
            let current_state_cloned = current_state.lock().get().clone();
            let speculative_state = current_state_cloned
                .current
                .freeze(&current_state_cloned.base);
            let latest_snapshot_state_view = CachedStateView::new_impl(
                StateViewId::Miscellaneous,
                num_transactions,
                snapshot,
                speculative_state,
                Arc::new(AsyncProofFetcher::new(state_db.clone())),
            );
            let write_sets = state_db
                .ledger_db
                .write_set_db()
                .get_write_sets(snapshot_next_version, num_transactions)?;
            let txn_info_iter = state_db
                .ledger_db
                .transaction_info_db()
                .get_transaction_info_iter(snapshot_next_version, write_sets.len())?;
            let last_checkpoint_index = txn_info_iter
                .into_iter()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .enumerate()
                .filter(|(_idx, txn_info)| txn_info.has_state_checkpoint_hash())
                .last()
                .map(|(idx, _)| idx);
            latest_snapshot_state_view.prime_cache_by_write_set(&write_sets)?;

            let state_checkpoint_output =
                InMemoryStateCalculatorV2::calculate_for_write_sets_after_snapshot(
                    &Arc::new(current_state_cloned),
                    &latest_snapshot_state_view.into_state_cache(),
                    last_checkpoint_index,
                    &write_sets,
                )?;

            // synchronously commit the snapshot at the last checkpoint here if not committed to disk yet.
            buffered_state.update(
                state_checkpoint_output
                    .state_updates_before_last_checkpoint
                    .as_ref(),
                &state_checkpoint_output.result_state,
                true, /* sync_commit */
            )?;
        }

        {
            let current_state = current_state.lock();
            info!(
                latest_snapshot_version = current_state.base_version,
                latest_snapshot_root_hash = current_state.base.root_hash(),
                latest_in_memory_version = current_state.current_version,
                latest_in_memory_root_hash = current_state.current.root_hash(),
                "StateStore initialization finished.",
            );
        }
        Ok(buffered_state)
        FIXME(aldenhu)
         */
        todo!()
    }

    pub fn reset(&self) {
        self.buffered_state.lock().drain();
        *self.buffered_state.lock() = Self::create_buffered_state_from_latest_snapshot(
            &self.state_db,
            self.buffered_state_target_items,
            false,
            true,
            self.current_state.clone(),
            self.persisted_state.clone(),
        )
        .expect("buffered state creation failed.");
    }

    pub fn buffered_state(&self) -> &Mutex<BufferedState> {
        &self.buffered_state
    }

    // FIXME(aldenhu): make current_state_cloned() this
    pub fn current_state(&self) -> MutexGuard<CurrentState> {
        self.current_state.lock()
    }

    pub fn current_state_cloned(&self) -> State {
        self.current_state().get().current.clone()
    }

    pub fn persisted_state(&self) -> MutexGuard<PersistedState> {
        self.persisted_state.lock()
    }

    /// Returns the key, value pairs for a particular state key prefix at at desired version. This
    /// API can be used to get all resources of an account by passing the account address as the
    /// key prefix.
    pub fn get_prefixed_state_value_iterator(
        &self,
        key_prefix: &StateKeyPrefix,
        first_key_opt: Option<&StateKey>,
        desired_version: Version,
    ) -> Result<PrefixedStateValueIterator> {
        // this can only handle non-sharded db scenario.
        // For sharded db, should look at API side using internal indexer to handle this request
        PrefixedStateValueIterator::new(
            &self.state_kv_db,
            key_prefix.clone(),
            first_key_opt.cloned(),
            desired_version,
        )
    }

    /// Gets the proof that proves a range of accounts.
    pub fn get_value_range_proof(
        &self,
        rightmost_key: HashValue,
        version: Version,
    ) -> Result<SparseMerkleRangeProof> {
        self.state_merkle_db.get_range_proof(rightmost_key, version)
    }

    /// Put the write sets on top of current state
    pub fn put_write_sets(
        &self,
        write_sets: Vec<WriteSet>,
        first_version: Version,
        batch: &SchemaBatch,
        sharded_state_kv_batches: &ShardedStateKvSchemaBatch,
        enable_sharding: bool,
    ) -> Result<()> {
        /*
        self.put_value_sets(
            first_version,
            &ShardedStateUpdateRefs::index_write_sets(&write_sets, write_sets.len()),
            StateStorageUsage::new_untracked(),
            None, // state cache
            batch,
            sharded_state_kv_batches,
            enable_sharding,
            None, // last_checkpoint_index
        )
        FIXME(aldenhu): need to calculate state, last checkpoint, etc
         */
        todo!()
    }

    /// Put the `value_state_sets` into its own CF.
    pub fn put_value_sets(
        &self,
        last_checkpoint_state: Option<&State>,
        state: &State,
        state_update_refs: &ShardedStateUpdateRefs,
        sharded_state_cache: Option<&ShardedStateCache>,
        ledger_batch: &SchemaBatch,
        sharded_state_kv_batches: &ShardedStateKvSchemaBatch,
        enable_sharding: bool,
        last_checkpoint_index: Option<usize>,
    ) -> Result<()> {
        let _timer = OTHER_TIMERS_SECONDS.timer_with(&["put_value_sets"]);
        let current_state = self.current_state().current.clone();

        self.put_stats_and_indices(
            &current_state,
            last_checkpoint_state,
            state,
            state_update_refs,
            sharded_state_cache,
            ledger_batch,
            sharded_state_kv_batches,
            last_checkpoint_index,
            enable_sharding,
        )?;

        self.put_state_values(
            current_state.next_version(),
            state_update_refs,
            sharded_state_kv_batches,
            enable_sharding,
        )
    }

    pub fn put_state_values(
        &self,
        first_version: Version,
        state_update_refs: &ShardedStateUpdateRefs,
        sharded_state_kv_batches: &ShardedStateKvSchemaBatch,
        enable_sharding: bool,
    ) -> Result<()> {
        let _timer = OTHER_TIMERS_SECONDS.timer_with(&["add_state_kv_batch"]);

        // TODO(aldenhu): put by refs; batch put
        sharded_state_kv_batches
            .par_iter()
            .zip_eq(state_update_refs.shards.par_iter())
            .try_for_each(|(batch, updates)| {
                updates.iter().try_for_each(|(idx, key, val)| {
                    let ver = first_version + *idx as Version;
                    if enable_sharding {
                        batch.put::<StateValueByKeyHashSchema>(
                            &(CryptoHash::hash(*key), ver),
                            &val.cloned(),
                        )
                    } else {
                        batch.put::<StateValueSchema>(&((*key).clone(), ver), &val.cloned())
                    }
                })
            })
    }

    pub fn get_usage(&self, version: Option<Version>) -> Result<StateStorageUsage> {
        let _timer = OTHER_TIMERS_SECONDS
            .with_label_values(&["get_usage"])
            .start_timer();
        self.state_db.get_state_storage_usage(version)
    }

    /// Put storage usage stats and State key and value indices into the batch.
    /// The state KV indices will be generated as follows:
    /// 1. A deletion at current version is always coupled with stale index for the tombstone with
    /// `stale_since_version` equal to the version, to ensure tombstone is cleared from db after
    /// pruner processes the current version.
    /// 2. An update at current version will first try to find the corresponding old value, if it
    /// exists, a stale index of that old value will be added. Otherwise, it's a no-op. Because
    /// non-existence means either the key never shows up or it got deleted. Neither case needs
    /// extra stale index as 1 cover the latter case.
    pub fn put_stats_and_indices(
        &self,
        current_state: &State,
        last_checkpoint_state: Option<&State>,
        state: &State,
        state_update_refs: &ShardedStateUpdateRefs,
        // If not None, it must contains all keys in the value_state_sets.
        // TODO(grao): Restructure this function.
        sharded_state_cache: Option<&ShardedStateCache>,
        batch: &SchemaBatch,
        sharded_state_kv_batches: &ShardedStateKvSchemaBatch,
        last_checkpoint_index: Option<usize>,
        enable_sharding: bool,
    ) -> Result<()> {
        let _timer = OTHER_TIMERS_SECONDS.timer_with(&["put_stats_and_indices"]);

        let _state_cache;
        let primed_state_cache = if let Some(cache) = sharded_state_cache {
            cache
        } else {
            // If no cache is provided, we load the old values of all keys inline.
            _state_cache = ShardedStateCache::default();
            self.prime_state_cache(current_state, state, &_state_cache);
            &_state_cache
        };

        Self::put_stale_state_value_index(
            current_state.next_version(),
            state_update_refs,
            sharded_state_kv_batches,
            enable_sharding,
            primed_state_cache,
            state.usage().is_untracked() || current_state.version().is_none(), // ignore_state_cache_miss
        );

        {
            let _timer = OTHER_TIMERS_SECONDS.timer_with(&["put_stats_and_indices__put_usage"]);
            if let Some(last_checkpoint_state) = last_checkpoint_state {
                if !last_checkpoint_state.is_the_same(state) {
                    Self::put_usage(last_checkpoint_state, batch)?;
                }
            }
            Self::put_usage(state, batch)?;
        }

        Ok(())
    }

    fn prime_state_cache(
        &self,
        current_state: &State,
        state: &State,
        state_cache: &ShardedStateCache,
    ) {
        if let Some(base_version) = current_state.version() {
            let _timer =
                OTHER_TIMERS_SECONDS.timer_with(&["put_stats_and_indices__prime_state_cache"]);

            state
                .clone()
                .into_delta(current_state.clone())
                .updates
                .shards
                .par_iter()
                .zip_eq(state_cache.shards.par_iter())
                .for_each(|(state_writes, cache)| {
                    for (key, _val) in state_writes.iter() {
                        let tuple_opt = self
                            .state_db
                            .get_state_value_with_version_by_version(&key, base_version)
                            .expect("Must succeed.");
                        cache.insert(key, StateValueWithVersionOpt::from_tuple_opt(tuple_opt));
                    }
                })
        }
    }

    fn put_stale_state_value_index(
        first_version: Version,
        state_update_refs: &ShardedStateUpdateRefs,
        sharded_state_kv_batches: &ShardedStateKvSchemaBatch,
        enable_sharding: bool,
        sharded_state_cache: &ShardedStateCache,
        ignore_state_cache_miss: bool,
    ) {
        let _timer = OTHER_TIMERS_SECONDS.timer_with(&["put_stale_kv_index"]);
        let num_versions = state_update_refs.num_versions;
        // calculate total state size in bytes
        sharded_state_cache
            .shards
            .par_iter()
            .zip_eq(state_update_refs.shards.par_iter())
            .zip_eq(sharded_state_kv_batches.par_iter())
            .enumerate()
            .for_each(|(shard_id, ((cache, updates), batch))| {
                Self::put_stale_state_value_index_for_shard(
                    shard_id,
                    first_version,
                    num_versions,
                    cache,
                    updates,
                    batch,
                    enable_sharding,
                    ignore_state_cache_miss,
                );
            })
    }

    fn put_stale_state_value_index_for_shard(
        shard_id: usize,
        first_version: Version,
        num_versions: usize,
        cache: &StateCacheShard,
        updates: &[StateUpdateRefWithOffset],
        batch: &SchemaBatch,
        enable_sharding: bool,
        ignore_state_cache_miss: bool,
    ) {
        let _timer = OTHER_TIMERS_SECONDS.timer_with(&[&format!("put_stale_kv_index__{shard_id}")]);

        let mut iter = updates.iter();
        for idx in 0..num_versions {
            let version = first_version + idx as Version;
            let ver_iter = iter.take_while_ref(|(i, _key, _val)| *i == idx);

            for (_idx, key, value) in ver_iter {
                if value.is_none() {
                    // Update the stale index of the tombstone at current version to
                    // current version.
                    if enable_sharding {
                        batch
                            .put::<StaleStateValueIndexByKeyHashSchema>(
                                &StaleStateValueByKeyHashIndex {
                                    stale_since_version: version,
                                    version,
                                    state_key_hash: key.hash(),
                                },
                                &(),
                            )
                            .unwrap();
                    } else {
                        batch
                            .put::<StaleStateValueIndexSchema>(
                                &StaleStateValueIndex {
                                    stale_since_version: version,
                                    version,
                                    state_key: (*key).clone(),
                                },
                                &(),
                            )
                            .unwrap();
                    }
                }

                let old_state_value_with_version_opt = if let Some(old) = cache.insert(
                    (*key).clone(),
                    StateValueWithVersionOpt::from_state_write_ref(version, *value),
                ) {
                    old
                } else {
                    // n.b. all updated state items must be read and recorded in the state cache,
                    // otherwise we can't calculate the correct usage. The is_untracked() hack
                    // is to allow some db tests without real execution layer to pass.
                    assert!(ignore_state_cache_miss, "Must cache read.");
                    StateValueWithVersionOpt::NonExistent
                };

                if let StateValueWithVersionOpt::Value {
                    version: old_version,
                    value: old_value,
                } = old_state_value_with_version_opt
                {
                    // stale index of the old value at its version.
                    if enable_sharding {
                        batch
                            .put::<StaleStateValueIndexByKeyHashSchema>(
                                &StaleStateValueByKeyHashIndex {
                                    stale_since_version: version,
                                    version: old_version,
                                    state_key_hash: key.hash(),
                                },
                                &(),
                            )
                            .unwrap();
                    } else {
                        batch
                            .put::<StaleStateValueIndexSchema>(
                                &StaleStateValueIndex {
                                    stale_since_version: version,
                                    version: old_version,
                                    state_key: (*key).clone(),
                                },
                                &(),
                            )
                            .unwrap();
                    }
                }
            }
        }
    }

    fn put_usage(state: &State, batch: &SchemaBatch) -> Result<()> {
        if let Some(version) = state.version() {
            let usage = state.usage();
            info!("Write usage at version {version}, {usage:?}.");
            batch.put::<VersionDataSchema>(&version, &usage.into())?;
        } else {
            assert_eq!(state.usage().items(), 0);
            assert_eq!(state.usage().bytes(), 0);
        }

        Ok(())
    }

    pub(crate) fn shard_state_value_batch(
        &self,
        sharded_batch: &ShardedStateKvSchemaBatch,
        values: &StateValueBatch,
        enable_sharding: bool,
    ) -> Result<()> {
        values.iter().for_each(|((key, version), value)| {
            let shard_id = key.get_shard_id() as usize;
            assert!(
                shard_id < NUM_STATE_SHARDS,
                "Invalid shard id: {}",
                shard_id
            );
            if enable_sharding {
                sharded_batch[shard_id]
                    .put::<StateValueByKeyHashSchema>(&(key.hash(), *version), value)
                    .expect("Inserting into sharded schema batch should never fail");
            } else {
                sharded_batch[shard_id]
                    .put::<StateValueSchema>(&(key.clone(), *version), value)
                    .expect("Inserting into sharded schema batch should never fail");
            }
        });
        Ok(())
    }

    /// Merklize the results generated by `value_state_sets` to `batch` and return the result root
    /// hashes for each write set.
    #[cfg(test)]
    pub fn merklize_value_set(
        &self,
        value_set: Vec<(HashValue, Option<&(HashValue, StateKey)>)>,
        version: Version,
        base_version: Option<Version>,
    ) -> Result<HashValue> {
        let (top_levels_batch, sharded_batch, root_hash) =
            self.state_merkle_db.merklize_value_set(
                value_set,
                version,
                base_version,
                /*previous_epoch_ending_version=*/ None,
            )?;
        self.state_merkle_db
            .commit(version, top_levels_batch, sharded_batch)?;
        Ok(root_hash)
    }

    pub fn get_root_hash(&self, version: Version) -> Result<HashValue> {
        self.state_merkle_db.get_root_hash(version)
    }

    pub fn get_value_count(&self, version: Version) -> Result<usize> {
        self.state_merkle_db.get_leaf_count(version)
    }

    pub fn get_state_key_and_value_iter(
        self: &Arc<Self>,
        version: Version,
        start_idx: usize,
    ) -> Result<impl Iterator<Item = Result<(StateKey, StateValue)>> + Send + Sync> {
        let store = Arc::clone(self);
        Ok(JellyfishMerkleIterator::new_by_index(
            Arc::clone(&self.state_merkle_db),
            version,
            start_idx,
        )?
        .map(|it| it.map_err(Into::into))
        .map(move |res| match res {
            Ok((_hashed_key, (key, version))) => {
                Ok((key.clone(), store.expect_value_by_version(&key, version)?))
            },
            Err(err) => Err(err),
        }))
    }

    pub fn get_value_chunk_with_proof(
        self: &Arc<Self>,
        version: Version,
        first_index: usize,
        chunk_size: usize,
    ) -> Result<StateValueChunkWithProof> {
        let result_iter = JellyfishMerkleIterator::new_by_index(
            Arc::clone(&self.state_merkle_db),
            version,
            first_index,
        )?
        .take(chunk_size)
        .map(|it| it.map_err(Into::into));
        let state_key_values: Vec<(StateKey, StateValue)> = result_iter
            .into_iter()
            .map(|res| {
                res.and_then(|(_, (key, version))| {
                    Ok((key.clone(), self.expect_value_by_version(&key, version)?))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            !state_key_values.is_empty(),
            "State chunk starting at {}",
            first_index,
        );
        let last_index = (state_key_values.len() - 1 + first_index) as u64;
        let first_key = state_key_values.first().expect("checked to exist").0.hash();
        let last_key = state_key_values.last().expect("checked to exist").0.hash();
        let proof = self.get_value_range_proof(last_key, version)?;
        let root_hash = self.get_root_hash(version)?;

        Ok(StateValueChunkWithProof {
            first_index: first_index as u64,
            last_index,
            first_key,
            last_key,
            raw_values: state_key_values,
            proof,
            root_hash,
        })
    }

    // state sync doesn't query for the progress, but keeps its record by itself.
    // TODO: change to async comment once it does like https://github.com/aptos-labs/aptos-core/blob/159b00f3d53e4327523052c1b99dd9889bf13b03/storage/backup/backup-cli/src/backup_types/state_snapshot/restore.rs#L147 or overlap at least two chunks.
    pub fn get_snapshot_receiver(
        self: &Arc<Self>,
        version: Version,
        expected_root_hash: HashValue,
    ) -> Result<Box<dyn StateSnapshotReceiver<StateKey, StateValue>>> {
        Ok(Box::new(StateSnapshotRestore::new(
            &self.state_merkle_db,
            self,
            version,
            expected_root_hash,
            false, /* async_commit */
            StateSnapshotRestoreMode::Default,
        )?))
    }

    #[cfg(test)]
    pub fn get_all_jmt_nodes_referenced(
        &self,
        version: Version,
    ) -> Result<Vec<aptos_jellyfish_merkle::node_type::NodeKey>> {
        aptos_jellyfish_merkle::JellyfishMerkleTree::new(self.state_merkle_db.as_ref())
            .get_all_nodes_referenced(version)
            .map_err(Into::into)
    }

    #[cfg(test)]
    pub fn get_all_jmt_nodes(&self) -> Result<Vec<aptos_jellyfish_merkle::node_type::NodeKey>> {
        let mut iter = self
            .state_db
            .state_merkle_db
            .metadata_db()
            .iter::<crate::schema::jellyfish_merkle_node::JellyfishMerkleNodeSchema>()?;
        iter.seek_to_first();

        let all_rows = iter.collect::<Result<Vec<_>>>()?;

        let mut keys: Vec<aptos_jellyfish_merkle::node_type::NodeKey> =
            all_rows.into_iter().map(|(k, _v)| k).collect();
        if self.state_merkle_db.sharding_enabled() {
            for i in 0..NUM_STATE_SHARDS as u8 {
                let mut iter =
                    self.state_merkle_db
                        .db_shard(i)
                        .iter::<crate::schema::jellyfish_merkle_node::JellyfishMerkleNodeSchema>()?;
                iter.seek_to_first();

                let all_rows = iter.collect::<Result<Vec<_>>>()?;
                keys.extend(all_rows.into_iter().map(|(k, _v)| k).collect::<Vec<_>>());
            }
        }
        Ok(keys)
    }
}

impl StateValueWriter<StateKey, StateValue> for StateStore {
    // This already turns on sharded KV
    fn write_kv_batch(
        &self,
        version: Version,
        node_batch: &StateValueBatch,
        progress: StateSnapshotProgress,
    ) -> Result<()> {
        let _timer = OTHER_TIMERS_SECONDS
            .with_label_values(&["state_value_writer_write_chunk"])
            .start_timer();
        let batch = SchemaBatch::new();
        let sharded_schema_batch = new_sharded_kv_schema_batch();

        batch.put::<DbMetadataSchema>(
            &DbMetadataKey::StateSnapshotKvRestoreProgress(version),
            &DbMetadataValue::StateSnapshotProgress(progress),
        )?;

        if self.internal_indexer_db.is_some()
            && self
                .internal_indexer_db
                .as_ref()
                .unwrap()
                .statekeys_enabled()
        {
            let keys = node_batch.iter().map(|(key, _)| key.0.clone()).collect();
            self.internal_indexer_db
                .as_ref()
                .unwrap()
                .write_keys_to_indexer_db(&keys, version, progress)?;
        }
        self.shard_state_value_batch(
            &sharded_schema_batch,
            node_batch,
            self.state_kv_db.enabled_sharding(),
        )?;
        self.state_kv_db
            .commit(version, batch, sharded_schema_batch)
    }

    fn kv_finish(&self, version: Version, usage: StateStorageUsage) -> Result<()> {
        self.ledger_db.metadata_db().put_usage(version, usage)?;
        if let Some(internal_indexer_db) = self.internal_indexer_db.as_ref() {
            if version > 0 {
                let batch = SchemaBatch::new();
                batch.put::<InternalIndexerMetadataSchema>(
                    &MetadataKey::LatestVersion,
                    &MetadataValue::Version(version - 1),
                )?;
                if internal_indexer_db.statekeys_enabled() {
                    batch.put::<InternalIndexerMetadataSchema>(
                        &MetadataKey::StateVersion,
                        &MetadataValue::Version(version - 1),
                    )?;
                }
                if internal_indexer_db.transaction_enabled() {
                    batch.put::<InternalIndexerMetadataSchema>(
                        &MetadataKey::TransactionVersion,
                        &MetadataValue::Version(version - 1),
                    )?;
                }
                if internal_indexer_db.event_enabled() {
                    batch.put::<InternalIndexerMetadataSchema>(
                        &MetadataKey::EventVersion,
                        &MetadataValue::Version(version - 1),
                    )?;
                }
                internal_indexer_db
                    .get_inner_db_ref()
                    .write_schemas(batch)?;
            }
        }

        Ok(())
    }

    fn get_progress(&self, version: Version) -> Result<Option<StateSnapshotProgress>> {
        let main_db_progress = self
            .state_kv_db
            .metadata_db()
            .get::<DbMetadataSchema>(&DbMetadataKey::StateSnapshotKvRestoreProgress(version))?
            .map(|v| v.expect_state_snapshot_progress());

        // verify if internal indexer db and main db are consistent before starting the restore
        if self.internal_indexer_db.is_some()
            && self
                .internal_indexer_db
                .as_ref()
                .unwrap()
                .statekeys_enabled()
        {
            let progress_opt = self
                .internal_indexer_db
                .as_ref()
                .unwrap()
                .get_restore_progress(version)?;

            match (main_db_progress, progress_opt) {
                (None, None) => (),
                (None, Some(_)) => (),
                (Some(main_progres), Some(indexer_progress)) => {
                    if main_progres.key_hash > indexer_progress.key_hash {
                        bail!(
                            "Inconsistent restore progress between main db and internal indexer db. main db: {:?}, internal indexer db: {:?}",
                            main_progres,
                            indexer_progress,
                        );
                    }
                },
                _ => {
                    bail!(
                        "Inconsistent restore progress between main db and internal indexer db. main db: {:?}, internal indexer db: {:?}",
                        main_db_progress,
                        progress_opt,
                    );
                },
            }
        }

        Ok(main_db_progress)
    }
}
