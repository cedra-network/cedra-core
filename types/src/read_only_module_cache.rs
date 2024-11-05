// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{error::PanicError, explicit_sync_wrapper::ExplicitSyncWrapper};
use crossbeam::utils::CachePadded;
use hashbrown::HashMap;
use move_vm_types::code::ModuleCode;
use std::{
    hash::Hash,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

/// Entry stored in [ReadOnlyModuleCache].
struct Entry<DC, VC, E> {
    /// True if this code is "valid" within the block execution context (i.e, there has been no
    /// republishing of this module so far). If false, executor needs to read the module from the
    /// sync/unsync module caches.
    valid: CachePadded<AtomicBool>,
    /// Cached verified module. While [ModuleCode] type is used, the following invariants always
    /// hold:
    ///    1. Module's version is [None] (storage version).
    ///    2. Module's code is always verified.
    module: CachePadded<Arc<ModuleCode<DC, VC, E, Option<u32>>>>,
}

impl<DC, VC, E> Entry<DC, VC, E>
where
    VC: Deref<Target = Arc<DC>>,
{
    /// Returns a new valid module. Returns a (panic) error if the module is not verified or has
    /// non-storage version.
    fn new(module: Arc<ModuleCode<DC, VC, E, Option<u32>>>) -> Result<Self, PanicError> {
        if !module.code().is_verified() || module.version().is_some() {
            let msg = format!(
                "Invariant violated for immutable module code : verified ({}), version({:?})",
                module.code().is_verified(),
                module.version()
            );
            return Err(PanicError::CodeInvariantError(msg));
        }

        Ok(Self {
            valid: CachePadded::new(AtomicBool::new(true)),
            module: CachePadded::new(module),
        })
    }

    /// Marks the module as invalid.
    fn mark_invalid(&self) {
        self.valid.store(false, Ordering::Release)
    }

    /// Returns true if the module is valid.
    pub(crate) fn is_valid(&self) -> bool {
        self.valid.load(Ordering::Acquire)
    }

    /// Returns the module code stored is this [Entry].
    fn inner(&self) -> &Arc<ModuleCode<DC, VC, E, Option<u32>>> {
        self.module.deref()
    }
}

/// A read-only module cache for verified code, that can be accessed concurrently within the block.
/// It can only be modified at block boundaries.
pub struct ReadOnlyModuleCache<K, DC, VC, E> {
    /// Module cache containing the verified code.
    module_cache: ExplicitSyncWrapper<HashMap<K, Entry<DC, VC, E>>>,
}

impl<K, DC, VC, E> ReadOnlyModuleCache<K, DC, VC, E>
where
    K: Hash + Eq + Clone,
    VC: Deref<Target = Arc<DC>>,
{
    /// Returns new empty module cache.
    pub fn empty() -> Self {
        Self {
            module_cache: ExplicitSyncWrapper::new(HashMap::new()),
        }
    }

    /// Returns true if the key exists in immutable cache and the corresponding module is valid.
    pub fn contains_valid(&self, key: &K) -> bool {
        self.module_cache
            .acquire()
            .get(key)
            .is_some_and(|module| module.is_valid())
    }

    /// Marks the cached module (if it exists) as invalid. As a result, all subsequent calls to the
    /// cache for the associated key  will result in a cache miss. Note that it is fine for an
    /// entry not to exist, in which case this is a no-op.
    pub fn mark_invalid(&self, key: &K) {
        if let Some(module) = self.module_cache.acquire().get(key) {
            module.mark_invalid();
        }
    }

    /// Returns the module stored in cache. If the module has not been cached, or it exists but is
    /// not valid, [None] is returned.
    pub fn get(&self, key: &K) -> Option<Arc<ModuleCode<DC, VC, E, Option<u32>>>> {
        self.module_cache.acquire().get(key).and_then(|module| {
            if module.is_valid() {
                Some(module.inner().clone())
            } else {
                None
            }
        })
    }

    /// Flushes the cache. Should never be called throughout block-execution. Use with caution.
    pub fn flush_unchecked(&self) {
        self.module_cache.acquire().clear();
    }

    /// Inserts modules into the cache. Should never be called throughout block-execution. Use with
    /// caution.
    ///
    /// Notes:
    ///   1. Only verified modules are inserted.
    ///   2. Versions of inserted modules are set to [None] (storage version).
    ///   3. Valid modules should not be removed, and new modules should have unique ownership. If
    ///      these constraints are violated, a panic error is returned.
    pub fn insert_verified_unchecked(
        &self,
        modules: impl Iterator<Item = (K, Arc<ModuleCode<DC, VC, E, Option<u32>>>)>,
    ) -> Result<(), PanicError> {
        use hashbrown::hash_map::Entry::*;

        let mut guard = self.module_cache.acquire();
        let module_cache = guard.dereference_mut();

        for (key, module) in modules {
            if let Occupied(entry) = module_cache.entry(key.clone()) {
                if entry.get().is_valid() {
                    return Err(PanicError::CodeInvariantError(
                        "Should never overwrite a valid module".to_string(),
                    ));
                } else {
                    // Otherwise, remove the invalid entry.
                    entry.remove();
                }
            }

            if module.code().is_verified() {
                let mut module = module.as_ref().clone();
                module.set_version(None);
                let prev = module_cache.insert(key.clone(), Entry::new(Arc::new(module))?);

                // At this point, we must have removed the entry, or returned a panic error.
                assert!(prev.is_none())
            }
        }
        Ok(())
    }

    /// Returns the size of the cache.
    pub fn size(&self) -> usize {
        self.module_cache.acquire().len()
    }

    /// Insert the module to cache. Used for tests only.
    #[cfg(any(test, feature = "testing"))]
    pub fn insert(&self, key: K, module: Arc<ModuleCode<DC, VC, E, Option<u32>>>) {
        self.module_cache
            .acquire()
            .insert(key, Entry::new(module).unwrap());
    }

    /// Removes the module from cache. Used for tests only.
    #[cfg(any(test, feature = "testing"))]
    pub fn remove(&self, key: &K) {
        self.module_cache.acquire().remove(key);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use claims::{assert_err, assert_ok, assert_some};
    use move_vm_types::code::{mock_deserialized_code, mock_verified_code};

    #[test]
    fn test_new_entry() {
        assert!(Entry::new(mock_deserialized_code(0, None)).is_err());
        assert!(Entry::new(mock_deserialized_code(0, Some(22))).is_err());
        assert!(Entry::new(mock_verified_code(0, Some(22))).is_err());
        assert!(Entry::new(mock_verified_code(0, None)).is_ok());
    }

    #[test]
    fn test_mark_entry_invalid() {
        let module_code = assert_ok!(Entry::new(mock_verified_code(0, None)));
        assert!(module_code.is_valid());

        module_code.mark_invalid();
        assert!(!module_code.is_valid());
    }

    #[test]
    fn test_get_entry() {
        let global_cache = ReadOnlyModuleCache::empty();

        global_cache.insert(0, mock_verified_code(0, None));
        global_cache.insert(1, mock_verified_code(1, None));
        global_cache.mark_invalid(&1);

        assert_eq!(global_cache.size(), 2);

        assert!(global_cache.contains_valid(&0));
        assert!(!global_cache.contains_valid(&1));
        assert!(!global_cache.contains_valid(&3));

        assert!(global_cache.get(&0).is_some());
        assert!(global_cache.get(&1).is_none());
        assert!(global_cache.get(&3).is_none());
    }

    #[test]
    fn test_insert_verified_for_read_only_module_cache() {
        let global_cache = ReadOnlyModuleCache::empty();

        let mut new_modules = vec![];
        for i in 0..10 {
            new_modules.push((i, mock_verified_code(i, Some(i as u32))));
        }
        let result = global_cache.insert_verified_unchecked(new_modules.into_iter());
        assert!(result.is_ok());
        assert_eq!(global_cache.size(), 10);

        // Versions should be set to storage.
        for key in 0..10 {
            let code = assert_some!(global_cache.get(&key));
            assert!(code.version().is_none())
        }

        global_cache.flush_unchecked();
        assert_eq!(global_cache.size(), 0);

        // Should not add deserialized code.
        let deserialized_modules = vec![(0, mock_deserialized_code(0, None))];
        assert_ok!(global_cache.insert_verified_unchecked(deserialized_modules.into_iter()));
        assert_eq!(global_cache.size(), 0);

        // Should not override valid modules.
        global_cache.insert(0, mock_verified_code(0, None));
        let new_modules = vec![(0, mock_verified_code(100, None))];
        assert_err!(global_cache.insert_verified_unchecked(new_modules.into_iter()));

        // Can override invalid modules.
        global_cache.mark_invalid(&0);
        let new_modules = vec![(0, mock_verified_code(100, None))];
        let result = global_cache.insert_verified_unchecked(new_modules.into_iter());
        assert!(result.is_ok());
        assert_eq!(global_cache.size(), 1);
    }
}
