// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use aptos_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    contract_event::ContractEvent, state_store::StateView,
};
use move_bytecode_utils::viewer::CompiledModuleViewer;
use move_core_types::language_storage::StructTag;
use move_resource_viewer::MoveValueAnnotator;
pub use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    ops::Deref,
};

pub struct AptosValueAnnotator<'a, T>(MoveValueAnnotator<'a, T>);

/// A wrapper around `MoveValueAnnotator` that adds a few aptos-specific functionalities.
#[derive(Debug)]
pub struct AnnotatedAccountStateBlob(BTreeMap<StructTag, AnnotatedMoveStruct>);

impl<'a, V: CompiledModuleViewer> AptosValueAnnotator<'a, V> {
    pub fn new(viewer: &'a V) -> Self {
        Self(MoveValueAnnotator::new(viewer))
    }

    pub fn view_resource(&self, tag: &StructTag, blob: &[u8]) -> Result<AnnotatedMoveStruct> {
        self.0.view_resource(tag, blob)
    }

    pub fn view_access_path(
        &self,
        access_path: AccessPath,
        blob: &[u8],
    ) -> Result<AnnotatedMoveStruct> {
        match access_path.get_struct_tag() {
            Some(tag) => self.view_resource(&tag, blob),
            None => bail!("Bad resource access path"),
        }
    }

    pub fn view_contract_event(&self, event: &ContractEvent) -> Result<AnnotatedMoveValue> {
        self.0.view_value(event.type_tag(), event.event_data())
    }

    pub fn view_account_state(&self, state: &AccountState) -> Result<AnnotatedAccountStateBlob> {
        let mut output = BTreeMap::new();
        for (k, v) in state.iter() {
            let tag = match AccessPath::new(AccountAddress::random(), k.to_vec()).get_struct_tag() {
                Some(t) => t,
                None => {
                    println!("Uncached AccessPath: {:?}", k);
                    continue;
                },
            };
            let value = self.view_resource(&tag, v)?;
            output.insert(tag, value);
        }
        Ok(AnnotatedAccountStateBlob(output))
    }
}

pub trait AsValueAnnotator<S> {
    fn as_value_annotator(&self) -> AptosValueAnnotator<S>;
}

impl<S: StateView + CompiledModuleViewer> AsValueAnnotator<S> for S {
    fn as_value_annotator(&self) -> AptosValueAnnotator<S> {
        AptosValueAnnotator::new(self)
    }
}

impl Display for AnnotatedAccountStateBlob {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for v in self.0.values() {
            write!(f, "{}", v)?;
            writeln!(f, ",")?;
        }
        writeln!(f, "}}")
    }
}

impl<'r, R> Deref for AptosValueAnnotator<'r, R> {
    type Target = MoveValueAnnotator<'r, R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
