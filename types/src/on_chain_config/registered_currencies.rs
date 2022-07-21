// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use anyhow::Result;
use move_deps::move_core_types::identifier::Identifier;
use serde::{Deserialize, Serialize};
use std::fmt;

// TODO: check whether this can be removed, as the resource does not exist (any longer)
// in the framework.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RegisteredCurrencies {
    currency_codes: Vec<Identifier>,
}

impl fmt::Display for RegisteredCurrencies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for currency_code in self.currency_codes().iter() {
            write!(f, "{} ", currency_code)?;
        }
        write!(f, "]")
    }
}

impl RegisteredCurrencies {
    pub fn currency_codes(&self) -> &[Identifier] {
        &self.currency_codes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl OnChainConfig for RegisteredCurrencies {
    // registered currencies address
    const MODULE_IDENTIFIER: &'static str = "registered_currencies";
    const TYPE_IDENTIFIER: &'static str = "RegisteredCurrencies";
}
