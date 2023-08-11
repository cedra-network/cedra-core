// Copyright © Aptos Foundation

use crate::v2::{PartitionState, PartitionerV2};
use aptos_types::{
    block_executor::partitioner::{
        PartitionedTransactions, SubBlock, SubBlocksForShard, TransactionWithDependencies,
    },
    transaction::analyzed_transaction::AnalyzedTransaction,
};
use rayon::{
    iter::ParallelIterator,
    prelude::{IntoParallelIterator, IntoParallelRefIterator},
};
use std::sync::Mutex;

impl PartitionerV2 {
    pub(crate) fn add_edges(state: &mut PartitionState) -> PartitionedTransactions {
        let mut sub_block_matrix: Vec<Vec<Mutex<Option<SubBlock<AnalyzedTransaction>>>>> =
            state.thread_pool.install(|| {
                (0..state.num_rounds())
                    .into_par_iter()
                    .map(|_round_id| {
                        (0..state.num_executor_shards)
                            .into_par_iter()
                            .map(|_shard_id| Mutex::new(None))
                            .collect()
                    })
                    .collect()
            });

        state.thread_pool.install(|| {
            (0..state.num_rounds())
                .into_par_iter()
                .for_each(|round_id| {
                    (0..state.num_executor_shards)
                        .into_par_iter()
                        .for_each(|shard_id| {
                            let twds = state.finalized_txn_matrix[round_id][shard_id]
                                .par_iter()
                                .map(|&ori_txn_idx| {
                                    state.make_txn_with_dep(round_id, shard_id, ori_txn_idx)
                                })
                                .collect();
                            let sub_block =
                                SubBlock::new(state.start_index_matrix[round_id][shard_id], twds);
                            *sub_block_matrix[round_id][shard_id].lock().unwrap() = Some(sub_block);
                        });
                });
        });

        let global_txns: Vec<TransactionWithDependencies<AnalyzedTransaction>> =
            if state.merge_discarded {
                sub_block_matrix
                    .pop()
                    .unwrap()
                    .last()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .take()
                    .unwrap()
                    .into_transactions_with_deps()
            } else {
                vec![]
            };

        let final_num_rounds = sub_block_matrix.len();
        let sharded_txns: Vec<SubBlocksForShard<AnalyzedTransaction>> = (0..state
            .num_executor_shards)
            .map(|shard_id| {
                let sub_blocks: Vec<SubBlock<AnalyzedTransaction>> = (0..final_num_rounds)
                    .map(|round_id| {
                        sub_block_matrix[round_id][shard_id]
                            .lock()
                            .unwrap()
                            .take()
                            .unwrap()
                    })
                    .collect();
                SubBlocksForShard::new(shard_id, sub_blocks)
            })
            .collect();
        let ret = PartitionedTransactions::new(sharded_txns, global_txns);

        state.thread_pool.spawn(move || {
            drop(sub_block_matrix);
        });
        ret
    }
}
