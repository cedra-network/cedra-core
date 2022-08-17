// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use aptos_aggregator::delta_change_set::{deserialize, DeltaOp};
use crossbeam::utils::CachePadded;
use dashmap::DashMap;
use std::{
    collections::btree_map::BTreeMap,
    fmt::Debug,
    hash::Hash,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

#[cfg(test)]
mod unit_tests;

// TODO: re-use definitions with the scheduler.
pub type TxnIndex = usize;
pub type Incarnation = usize;
pub type Version = (TxnIndex, Incarnation);

const FLAG_DONE: usize = 0;
const FLAG_ESTIMATE: usize = 1;

/// Type of entry, recorded in the shared multi-version data-structure for each write/delta.
struct Entry<V, D> {
    /// Used to mark the entry as a "write/delta estimate".
    flag: AtomicUsize,
    /// Write/delta data stored.
    inner: Cell<V, D>,
}

enum Cell<V, D> {
    WriteCell {
        /// Incarnation number of the transaction that wrote the entry. Note that
        /// TxnIndex is part of the key and not recorded here.
        incarnation: Incarnation,
        /// Actual data stored in a shared pointer (to ensure ownership and avoid clones).
        data: Arc<V>,
    },
    DeltaCell {
        /// Delta stored as a value.
        data: D,
    },
}

impl<V, D> Entry<V, D> {
    pub fn new_write_from(flag: usize, incarnation: Incarnation, data: V) -> Entry<V, D> {
        Entry {
            flag: AtomicUsize::new(flag),
            inner: Cell::WriteCell {
                incarnation,
                data: Arc::new(data),
            },
        }
    }

    pub fn new_delta_from(flag: usize, data: D) -> Entry<V, D> {
        Entry {
            flag: AtomicUsize::new(flag),
            inner: Cell::DeltaCell { data },
        }
    }

    pub fn flag(&self) -> usize {
        self.flag.load(Ordering::SeqCst)
    }

    pub fn mark_estimate(&self) {
        self.flag.store(FLAG_ESTIMATE, Ordering::SeqCst);
    }
}

/// Main multi-version data-structure used by threads to read, write, or apply deltas
/// during parallel execution. Maps each access path to an interal BTreeMap that
/// contains the indices of transactions that write or update at the given access path
/// alongside the corresponding entries of WriteCell or DeltaCell types.
/// Concurrency is managed by DashMap, i.e. when a method accesses a BTreeMap at a
/// given key, it holds exclusive access and doesn't need to explicitly synchronize
/// with other reader/writers.
pub struct MVHashMap<K, V> {
    data: DashMap<K, BTreeMap<TxnIndex, CachePadded<Entry<V, DeltaOp>>>>,
}

/// Error type returned when reading from the multi-version data-structure.
#[derive(Debug, PartialEq, Eq)]
pub enum MVHashMapError<D> {
    /// No prior entry is found.
    EntryNotFound,
    /// Read resulted in an unresolved delta value.
    UnresolvedDelta(D),
    /// A dependency on other transaction has been found during the read.
    Dependency(TxnIndex),
}

/// Output returned when reading from the multi-version data-structure.
#[derive(Debug, PartialEq, Eq)]
pub enum MVHashMapOutput<V> {
    /// Value which is the result of delta application, always a u128.
    ResolvedDelta(u128),
    /// Information from the last versioned-write.
    Versioned(Version, Arc<V>),
}

pub type MVHashMapResult<V, D> = Result<MVHashMapOutput<V>, MVHashMapError<D>>;

impl<K: Hash + Clone + Eq, V: AsRef<Vec<u8>>> MVHashMap<K, V> {
    pub fn new() -> MVHashMap<K, V> {
        MVHashMap {
            data: DashMap::new(),
        }
    }

    /// Write a versioned data at a specified key. If the WriteCell entry is overwritten,
    /// asserts that the new incarnation is strictly higher.
    pub fn write(&self, key: &K, version: Version, data: V) {
        let (txn_idx, incarnation) = version;

        let mut map = self.data.entry(key.clone()).or_insert(BTreeMap::new());
        let prev_entry = map.insert(
            txn_idx,
            CachePadded::new(Entry::new_write_from(FLAG_DONE, incarnation, data)),
        );

        // Assert that the previous entry for txn_idx, if present, had lower incarnation.
        assert!(prev_entry
            .map(|entry| matches!(&entry.inner, Cell::WriteCell { incarnation: i, data: _ } if *i < incarnation))
            .unwrap_or(true));
    }

    /// Mark an entry from transaction 'txn_idx' at access path 'key' as an estimated write
    /// (for future incarnation). Will panic if the entry is not in the data-structure.
    pub fn mark_estimate(&self, key: &K, txn_idx: TxnIndex) {
        let map = self.data.get(key).expect("Path must exist");
        map.get(&txn_idx)
            .expect("Entry by txn must exist")
            .mark_estimate();
    }

    /// Delete an entry from transaction 'txn_idx' at access path 'key'. Will panic
    /// if the access path has never been written before.
    pub fn delete(&self, key: &K, txn_idx: TxnIndex) {
        // TODO: investigate logical deletion.
        let mut map = self.data.get_mut(key).expect("Path must exist");
        map.remove(&txn_idx);
    }

    /// If successful, returns a read value or its version. Otherwise an error
    /// is returned.
    pub fn read(&self, key: &K, txn_idx: TxnIndex) -> MVHashMapResult<V, DeltaOp> {
        use MVHashMapError::*;
        use MVHashMapOutput::*;

        match self.data.get(key) {
            Some(tree) => {
                let mut iter = tree.range(0..txn_idx);
                let mut aggregated: Option<DeltaOp> = None;

                // Because read can hit a delta, we need to keep reading until we
                // reach a write or have to check storage.
                while let Some((idx, entry)) = iter.next_back() {
                    let flag = entry.flag();

                    if flag == FLAG_ESTIMATE {
                        // Found a dependency.
                        return Err(Dependency(*idx));
                    } else {
                        // The entry should be populated.
                        debug_assert!(flag == FLAG_DONE);

                        use Cell::*;
                        match &entry.inner {
                            WriteCell { incarnation, data } => {
                                match aggregated {
                                    // Read hits a write without any aggregation. In this
                                    // case simply return the entry.
                                    None => {
                                        let write_version = (*idx, *incarnation);
                                        return Ok(Versioned(write_version, data.clone()));
                                    }
                                    // Read hits a write during data aggregation. Apply aggregated value.
                                    Some(delta) => {
                                        // TODO: change this once trait is available!
                                        let base = deserialize(data.as_ref().as_ref());

                                        match delta.apply_to(base) {
                                            Err(_) => panic!("overflow!"),
                                            Ok(v) => {
                                                return Ok(ResolvedDelta(v));
                                            }
                                        }
                                    }
                                }
                            }
                            DeltaCell { data } => {
                                match aggregated {
                                    // Read hits a delta value during data aggregation.
                                    // Update the currently aggregated value.
                                    Some(delta) => match delta.merge_with(data.clone()) {
                                        Err(_) => panic!("terrible thing"),
                                        Ok(d) => aggregated = Some(d),
                                    },
                                    // Read hits a delta value and has to start data
                                    // aggregation. Initialize the aggregated value.
                                    None => aggregated = Some(data.clone()),
                                }
                            }
                        }
                    }
                }

                // It can happen that while resolving deltas the actual written value has not
                // been seen yet (i.e. added as an entry to the data-structure). In that case,
                // user calling read() must resolve delta with a value from the storage.
                match aggregated {
                    Some(delta) => Err(UnresolvedDelta(delta)),
                    None => Err(EntryNotFound),
                }
            }
            None => Err(EntryNotFound),
        }
    }
}
