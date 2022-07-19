// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_deps::{
    move_binary_format::file_format::CompiledModule,
    move_compiler::{
        compiled_unit::{CompiledUnit, NamedCompiledModule},
        shared::NumericalAddress,
    },
    move_package::compilation::compiled_package::CompiledPackage,
};
use once_cell::sync::Lazy;
use std::collections::BTreeMap;

const APTOS_FRAMEWORK_DIR: &str = "aptos-framework/sources";
const APTOS_STDLIB_DIR: &str = "aptos-stdlib/sources";
const MOVE_STDLIB_DIR: &str = "move-stdlib/sources";
const TOKEN_MODULES_DIR: &str = "aptos-token/sources";
static APTOS_PKG: Lazy<CompiledPackage> = Lazy::new(|| super::package("aptos-framework"));
static TOKEN_PKG: Lazy<CompiledPackage> = Lazy::new(|| super::package("aptos-token"));

pub fn files() -> Vec<String> {
    let mut files = super::move_files_in_path(MOVE_STDLIB_DIR);
    files.extend(super::move_files_in_path(APTOS_STDLIB_DIR));
    files.extend(super::move_files_in_path(APTOS_FRAMEWORK_DIR));
    files.extend(super::move_files_in_path(TOKEN_MODULES_DIR));
    files.extend(super::move_files_in_path(APTOS_STDLIB_DIR));

    files
}

pub fn module_blobs() -> Vec<Vec<u8>> {
    let mut framework_blobs = super::module_blobs(&*APTOS_PKG);
    framework_blobs.extend(super::module_blobs(&*TOKEN_PKG));
    framework_blobs
}

pub fn named_addresses() -> BTreeMap<String, NumericalAddress> {
    let mut framework = super::named_addresses(&*APTOS_PKG);
    framework.append(&mut super::named_addresses(&*TOKEN_PKG));
    framework
}

pub fn modules() -> Vec<CompiledModule> {
    let mut framework: Vec<CompiledModule> = APTOS_PKG
        .all_compiled_units()
        .filter_map(|unit| match unit {
            CompiledUnit::Module(NamedCompiledModule { module, .. }) => Some(module.clone()),
            CompiledUnit::Script(_) => None,
        })
        .collect();
    framework.extend(
        TOKEN_PKG
            .all_compiled_units()
            .filter_map(|unit| match unit {
                CompiledUnit::Module(NamedCompiledModule { module, .. }) => Some(module.clone()),
                CompiledUnit::Script(_) => None,
            })
            .collect::<Vec<CompiledModule>>(),
    );
    framework
}
