// Copyright (c) Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// Copyright © Aptos Foundation

use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

/// For each key, maintains a set of reservations and a set of pending requests,
/// each identified by a transaction ID.
/// A request for a key is satisfied if there are no reservations with smaller `TxnId`.
/// When a reservation is removed, returns the set of satisfied requests and removes them
/// from the reservation table.
pub trait ReservationTable {
    type Key;
    type TxnId: Copy;

    /// Adds a reservation to the table.
    fn make_reservation(&mut self, idx: Self::TxnId, key: &Self::Key);

    /// Adds reservations to the table.
    fn make_reservations<'a, KS>(&mut self, idx: Self::TxnId, keys: KS)
    where
        Self::Key: 'a,
        KS: IntoIterator<Item = &'a Self::Key>,
    {
        for key in keys.into_iter() {
            self.make_reservation(idx, key);
        }
    }

    /// Tries to add a pending request to the table.
    /// Returns `true` if a pending request is added and false if it is already satisfied.
    fn make_request(&mut self, idx: Self::TxnId, key: &Self::Key) -> bool;

    /// Tries to add pending requests to the table.
    /// Returns the number of added pending requests (i.e., the number of requests in the input
    /// that are not already satisfied).
    fn make_requests<'a, KS>(&mut self, idx: Self::TxnId, keys: KS) -> usize
    where
        Self::Key: 'a,
        KS: IntoIterator<Item = &'a Self::Key>,
    {
        keys.into_iter()
            .filter(|&key| self.make_request(idx, key))
            .count()
    }

    /// Checks whether the request is satisfied without adding it as a pending request
    /// if it isn't.
    fn is_satisfied(&self, idx: Self::TxnId, key: &Self::Key) -> bool;

    /// Checks whether all the requests are satisfied without adding them as pending requests
    /// if they aren't.
    fn are_all_satisfied<'a, KS>(&self, idx: Self::TxnId, keys: KS) -> bool
    where
        Self::Key: 'a,
        KS: IntoIterator<Item = &'a Self::Key>,
    {
        keys.into_iter().all(|key| self.is_satisfied(idx, key))
    }

    // TODO: consider returning an iterator instead of a vector
    /// Removes a reservation from the table and returns the requests that
    /// are now satisfied and removes them from the table.
    fn remove_reservation(
        &mut self,
        idx: Self::TxnId,
        keys: &Self::Key,
    ) -> Vec<(Self::TxnId, Self::Key)>;

    // TODO: consider returning an iterator instead of a vector
    /// Removes reservations from the table and returns the requests that
    /// are now satisfied and removes them from the table.
    fn remove_reservations<'a, KS>(
        &mut self,
        idx: Self::TxnId,
        keys: KS,
    ) -> Vec<(Self::TxnId, Self::Key)>
    where
        Self::Key: 'a,
        KS: IntoIterator<Item = &'a Self::Key>,
    {
        let mut res = Vec::new();
        for key in keys.into_iter() {
            res.append(&mut self.remove_reservation(idx, key));
        }
        res
    }
}

#[derive(Default)]
struct TableEntry<Id> {
    reservations: VecDeque<Id>,
    requests: VecDeque<Id>,
    delayed_removes: HashSet<Id>,
}

pub struct HashMapReservationTable<K, Id>(HashMap<K, TableEntry<Id>>);

impl<K, Id> Default for HashMapReservationTable<K, Id> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<K, Id> ReservationTable for HashMapReservationTable<K, Id>
where
    K: Hash + Eq + Clone,
    Id: Ord + Eq + Hash + Default + Copy,
{
    type Key = K;
    type TxnId = Id;

    fn make_reservation(&mut self, idx: Id, key: &K) {
        let reservations = &mut self.0.entry(key.clone()).or_default().reservations;
        debug_assert!(reservations.back().map_or(true, |&last| last <= idx));
        reservations.push_back(idx);
    }

    fn make_request(&mut self, idx: Id, key: &K) -> bool {
        if !self.is_satisfied(idx, key) {
            let requests = &mut self.0.get_mut(key).unwrap().requests;
            debug_assert!(requests.back().map_or(true, |&last| last <= idx));
            self.0.get_mut(key).unwrap().requests.push_back(idx);
            true
        } else {
            false
        }
    }

    fn is_satisfied(&self, idx: Id, request_key: &K) -> bool {
        let Some(entry) = self.0.get(request_key) else { return true };
        let Some(&first_reservation) = entry.reservations.front() else { return true };
        first_reservation >= idx
    }

    fn remove_reservation(&mut self, idx: Id, key: &K) -> Vec<(Id, K)> {
        let entry = self.0.get_mut(key).unwrap();
        entry.delayed_removes.insert(idx);

        while let Some(&first_reservation) = entry.reservations.front() {
            if !entry.delayed_removes.contains(&first_reservation) {
                break;
            }
            entry.delayed_removes.remove(&first_reservation);
            entry.reservations.pop_front();
        }

        match entry.reservations.front() {
            None => {
                let res = entry
                    .requests
                    .iter()
                    .map(|&req_idx| (req_idx, key.clone()))
                    .collect();
                entry.requests.clear();
                res
            },
            Some(first_reservation) => {
                let mut res = Vec::new();

                while let Some(&first_request) = entry.requests.front() {
                    if first_request > *first_reservation {
                        break;
                    }
                    res.push((entry.requests.pop_front().unwrap(), key.clone()));
                }

                res
            },
        }
    }
}