// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{TransactionGenerator, TransactionGeneratorCreator};
use aptos_sdk::types::{transaction::SignedTransaction, LocalAccount};
use std::sync::{atomic::AtomicU64, Arc};

struct BoundedBatchWrapperTransactionGenerator {
    batch_size: usize,
    generator: Box<dyn TransactionGenerator>,
}

impl TransactionGenerator for BoundedBatchWrapperTransactionGenerator {
    fn generate_transactions(
        &mut self,
        account: &LocalAccount,
        num_to_create: usize,
        _history: &[String],
        _market_maker: bool,
    ) -> Vec<SignedTransaction> {
        self.generator.generate_transactions(
            account,
            num_to_create.min(self.batch_size),
            &Vec::new(),
            false,
        )
    }
}

pub struct BoundedBatchWrapperTransactionGeneratorCreator {
    batch_size: usize,
    generator_creator: Box<dyn TransactionGeneratorCreator>,
}

impl BoundedBatchWrapperTransactionGeneratorCreator {
    #[allow(unused)]
    pub fn new(batch_size: usize, generator_creator: Box<dyn TransactionGeneratorCreator>) -> Self {
        Self {
            batch_size,
            generator_creator,
        }
    }
}

impl TransactionGeneratorCreator for BoundedBatchWrapperTransactionGeneratorCreator {
    fn create_transaction_generator(
        &self,
        txn_counter: Arc<AtomicU64>,
    ) -> Box<dyn TransactionGenerator> {
        Box::new(BoundedBatchWrapperTransactionGenerator {
            batch_size: self.batch_size,
            generator: self
                .generator_creator
                .create_transaction_generator(txn_counter),
        })
    }
}
