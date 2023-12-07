// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::error::QuorumStoreError;
use aptos_consensus_types::common::{Payload, PayloadFilter};
use aptos_types::validator_txn::{pool::ValidatorTransactionFilter, ValidatorTransaction};
use futures::future::BoxFuture;
use std::time::Duration;

pub mod mixed;
pub mod user;
#[cfg(test)]
pub mod validator;

#[async_trait::async_trait]
pub trait PayloadClient: Send + Sync {
    async fn pull_payload(
        &self,
        max_poll_time: Duration,
        max_items: u64,
        max_bytes: u64,
        validator_txn_filter: ValidatorTransactionFilter,
        user_txn_filter: PayloadFilter,
        wait_callback: BoxFuture<'static, ()>,
        pending_ordering: bool,
        pending_uncommitted_blocks: usize,
        recent_max_fill_fraction: f32,
    ) -> anyhow::Result<(Vec<ValidatorTransaction>, Payload), QuorumStoreError>;

    fn trace_payloads(&self) {}
}
