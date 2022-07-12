// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::U64;

use aptos_types::{chain_id::ChainId, ledger_info::LedgerInfoWithSignatures};
use poem_openapi::Object as PoemObject;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, PoemObject)]
pub struct LedgerInfo {
    pub chain_id: u8,
    pub epoch: u64,
    pub ledger_version: U64,
    pub oldest_ledger_version: U64,
    pub ledger_timestamp: U64,
}

impl LedgerInfo {
    pub fn new(
        chain_id: &ChainId,
        info: &LedgerInfoWithSignatures,
        oldest_ledger_version: u64,
    ) -> Self {
        let ledger_info = info.ledger_info();
        Self {
            chain_id: chain_id.id(),
            epoch: ledger_info.epoch(),
            ledger_version: ledger_info.version().into(),
            oldest_ledger_version: oldest_ledger_version.into(),
            ledger_timestamp: ledger_info.timestamp_usecs().into(),
        }
    }

    pub fn version(&self) -> u64 {
        self.ledger_version.into()
    }

    pub fn timestamp(&self) -> u64 {
        self.ledger_timestamp.into()
    }
}
