// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    transaction::BlockExecutableTransaction as Transaction,
    txn_provider::{TxnIndex, TxnProvider},
};

pub struct DefaultTxnProvider<T: Transaction> {
    pub txns: Vec<T>,
}

impl<T: Transaction> DefaultTxnProvider<T> {
    pub fn new(txns: Vec<T>) -> Self {
        Self { txns }
    }

    pub fn get_txns(&self) -> &Vec<T> {
        &self.txns
    }
}

impl<T: Transaction> TxnProvider<T> for DefaultTxnProvider<T> {
    fn num_txns(&self) -> usize {
        self.txns.len()
    }

    fn get_txn(&self, idx: TxnIndex) -> &T {
        &self.txns[idx as usize]
    }

    fn to_vec(&self) -> Vec<T> {
        self.txns.clone()
    }
}

impl<T: Transaction> Iterator for DefaultTxnProvider<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.txns.pop()
    }
}