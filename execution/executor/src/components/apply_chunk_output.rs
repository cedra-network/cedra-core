// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    components::{
        in_memory_state_calculator_v2::InMemoryStateCalculatorV2,
    },
    metrics::{OTHER_TIMERS},
};
use anyhow::{Result};
use aptos_crypto::{hash::CryptoHash, HashValue};
use aptos_executor_types::{
    parsed_transaction_output::TransactionsWithParsedOutput,
    should_forward_to_subscription_service,
    state_checkpoint_output::{StateCheckpointOutput},
    LedgerUpdateOutput, ParsedTransactionOutput,
};
use aptos_experimental_runtimes::thread_manager::optimal_min_len;
use aptos_metrics_core::TimerHelper;
use aptos_storage_interface::{state_delta::StateDelta, };
use aptos_types::{
    contract_event::ContractEvent,
    proof::accumulator::{InMemoryEventAccumulator, InMemoryTransactionAccumulator},
    transaction::{
        TransactionInfo,
    },
};
use itertools::{izip, Itertools};
use rayon::prelude::*;
use std::{sync::Arc};
use aptos_executor_types::chunk_output::ChunkOutput;

pub struct ApplyChunkOutput;

impl ApplyChunkOutput {
    pub fn calculate_state_checkpoint(
        chunk_output: &ChunkOutput,
        parent_state: &StateDelta,
        known_state_checkpoints: Option<Vec<Option<HashValue>>>,
        is_block: bool,
    ) -> Result<StateCheckpointOutput> {
        // Apply the write set, get the latest state.
        let mut res = InMemoryStateCalculatorV2::calculate_for_transactions(
            parent_state,
            chunk_output,
            is_block,
        )?;

        // On state sync/replay, we generate state checkpoints only periodically, for the
        // last state checkpoint of each chunk.
        // A mismatch in the SMT will be detected at that occasion too. Here we just copy
        // in the state root from the TxnInfo in the proof.
        if let Some(state_checkpoint_hashes) = known_state_checkpoints {
            res.check_and_update_state_checkpoint_hashes(state_checkpoint_hashes)?;
        }

        Ok(res)
    }

    pub fn calculate_ledger_update(
        chunk_output: &ChunkOutput,
        state_checkpoint_hashes: &[Option<HashValue>],
        base_txn_accumulator: &InMemoryTransactionAccumulator,
    ) -> Result<LedgerUpdateOutput> {
        let _timer = OTHER_TIMERS.timer_with(&["assemble_ledger_diff_for_block"]);

        // Update counters.
        chunk_output.update_counters_for_processed_chunk();

        // Calculate hashes
        let to_commit = &chunk_output.to_commit;
        let txn_outs = to_commit.parsed_outputs();

        let (event_hashes, writeset_hashes) = Self::calculate_events_and_writeset_hashes(txn_outs);

        // Assemble `TransactionInfo`s
        let (transaction_infos, subscribable_events) = Self::assemble_transaction_infos(
            &to_commit,
            &state_checkpoint_hashes,
            &event_hashes,
            &writeset_hashes,
        );

        // Calculate root hash
        let transaction_info_hashes = transaction_infos.iter().map(CryptoHash::hash).collect_vec();
        let transaction_accumulator =
            Arc::new(base_txn_accumulator.append(&transaction_info_hashes));

        Ok(LedgerUpdateOutput {
            transaction_infos,
            transaction_info_hashes,
            transaction_accumulator,
            subscribable_events,
        })
    }

    /* FIXME(aldenhu): remove
    pub fn apply_chunk(
        chunk_output: ChunkOutput,
        base_view: &ExecutedTrees,
        known_state_checkpoint_hashes: Option<Vec<Option<HashValue>>>,
    ) -> Result<(ExecutedChunk, Vec<Transaction>, Vec<Transaction>)> {
        let (result_state, next_epoch_state, state_checkpoint_output) =
            Self::calculate_state_checkpoint(
                chunk_output,
                base_view.state(),
                None, // append_state_checkpoint_to_block
                known_state_checkpoint_hashes,
                /*is_block=*/ false,
            )?;
        let (ledger_update_output, to_discard, to_retry) = Self::calculate_ledger_update(
            state_checkpoint_output,
            base_view.txn_accumulator().clone(),
        )?;

        Ok((
            ExecutedChunk {
                result_state,
                ledger_info: None,
                next_epoch_state,
                ledger_update_output,
            },
            to_discard,
            to_retry,
        ))
    }
     */

    fn assemble_transaction_infos(
        to_commit: &TransactionsWithParsedOutput,
        state_checkpoint_hashes: &[Option<HashValue>],
        event_hashes: &[HashValue],
        writeset_hashes: &[HashValue],
    ) -> (Vec<TransactionInfo>, Vec<ContractEvent>) {
        let _timer = OTHER_TIMERS.timer_with(&["process_events_and_writeset_hashes"]);

        izip!(
            to_commit.iter(),
            state_checkpoint_hashes,
            event_hashes,
            writeset_hashes
        )
        .map(
            |((txn, txn_out), state_checkpoint_hash, event_root_hash, write_set_hash)| {
                let subscribable_events: Vec<ContractEvent> = txn_out
                    .events()
                    .iter()
                    .filter(should_forward_to_subscription_service)
                    .cloned()
                    .collect();
                let txn_info = TransactionInfo::new(
                    txn.hash(),
                    write_set_hash,
                    event_root_hash,
                    state_checkpoint_hash.cloned(),
                    txn_out.gas_used(),
                    txn_out.status().as_kept_status().expect("Already sorted."),
                );
                (txn_info, subscribable_events)
            },
        )
        .unzip()
    }

    fn calculate_events_and_writeset_hashes(
        to_commit: &[ParsedTransactionOutput],
    ) -> (Vec<HashValue>, Vec<HashValue>) {
        let _timer = OTHER_TIMERS.timer_with(&["calculate_events_and_writeset_hashes"]);

        let num_txns = to_commit.len();
        to_commit
            .par_iter()
            .with_min_len(optimal_min_len(num_txns, 64))
            .map(|txn_output| {
                let event_hashes = txn_output
                    .events()
                    .iter()
                    .map(CryptoHash::hash)
                    .collect::<Vec<_>>();

                (
                    InMemoryEventAccumulator::from_leaves(&event_hashes).root_hash(),
                    CryptoHash::hash(txn_output.write_set()),
                )
            })
            .unzip()
    }
}

mod tests {
    #[test]
    fn assemble_ledger_diff_should_filter_subscribable_events() {
        let event_0 =
            ContractEvent::new_v2_with_type_tag_str("0x1::dkg::DKGStartEvent", b"dkg_1".to_vec());
        let event_1 = ContractEvent::new_v2_with_type_tag_str(
            "0x2345::random_module::RandomEvent",
            b"random_x".to_vec(),
        );
        let event_2 =
            ContractEvent::new_v2_with_type_tag_str("0x1::dkg::DKGStartEvent", b"dkg_2".to_vec());
        let txns_n_outputs =
            TransactionsWithParsedOutput::new(vec![Transaction::dummy(), Transaction::dummy()], vec![
                ParsedTransactionOutput::from(TransactionOutput::new(
                    WriteSet::default(),
                    vec![event_0.clone()],
                    0,
                    TransactionStatus::Keep(ExecutionStatus::Success),
                    TransactionAuxiliaryData::default(),
                )),
                ParsedTransactionOutput::from(TransactionOutput::new(
                    WriteSet::default(),
                    vec![event_1.clone(), event_2.clone()],
                    0,
                    TransactionStatus::Keep(ExecutionStatus::Success),
                    TransactionAuxiliaryData::default(),
                )),
            ]);
        let state_updates_vec = vec![
            ShardedStateUpdates::default(),
            ShardedStateUpdates::default(),
        ];
        let state_checkpoint_hashes = vec![Some(HashValue::zero()), Some(HashValue::zero())];
        let (_, _, subscribable_events) = ApplyChunkOutput::calculate_transaction_infos(
            txns_n_outputs,
            state_updates_vec,
            state_checkpoint_hashes,
        );
        assert_eq!(vec![event_0, event_2], subscribable_events);
    }
}
