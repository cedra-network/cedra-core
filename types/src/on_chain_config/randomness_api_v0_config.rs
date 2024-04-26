// Copyright (c) Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{move_utils::as_move_value::AsMoveValue, on_chain_config::OnChainConfig};
use move_core_types::value::{MoveStruct, MoveValue};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RequiredGasDeposit {
    pub gas_amount: Option<u64>,
}

impl RequiredGasDeposit {
    pub fn default_for_genesis() -> Self {
        Self {
            gas_amount: Some(1_000_000),
        }
    }

    pub fn default_if_missing() -> Self {
        Self { gas_amount: None }
    }
}

impl OnChainConfig for RequiredGasDeposit {
    const MODULE_IDENTIFIER: &'static str = "randomness_api_v0_config";
    const TYPE_IDENTIFIER: &'static str = "RequiredGasDeposit";
}

impl AsMoveValue for RequiredGasDeposit {
    fn as_move_value(&self) -> MoveValue {
        MoveValue::Struct(MoveStruct::Runtime(vec![self.gas_amount.as_move_value()]))
    }
}
