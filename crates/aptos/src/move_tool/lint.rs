// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::types::{AccountAddressWrapper, CliCommand, CliTypedResult, MovePackageDir},
    move_tool::IncludedArtifacts,
};
use aptos_framework::{BuildOptions, BuiltPackage};
use async_trait::async_trait;
use clap::Parser;
use move_compiler_v2::Experiment;
use move_model::metadata::{CompilerVersion, LanguageVersion};
use move_package::source_package::std_lib::StdVersion;
use std::{collections::BTreeMap, path::PathBuf};

/// Run a Lint tool to show additional warnings about the current package, in addition to ordinary
/// warnings and/or errors generated by the Move 2 compiler.
#[derive(Debug, Clone, Parser)]
pub struct LintPackage {
    /// Path to a move package (the folder with a Move.toml file).  Defaults to current directory.
    #[clap(long, value_parser)]
    pub package_dir: Option<PathBuf>,

    /// Specify the path to save the compiled bytecode files which lint generates while
    /// running checks.
    /// Defaults to `<package_dir>/build`
    #[clap(long, value_parser)]
    pub output_dir: Option<PathBuf>,

    ///     or `--language <LANGUAGE_VERSION>`
    /// Specify the language version to be supported.
    /// Currently, defaults to `2.0`.
    #[clap(long, value_parser = clap::value_parser!(LanguageVersion),
           alias = "language",
           default_value = "2.0",
           verbatim_doc_comment)]
    pub language_version: Option<LanguageVersion>,

    /// Named addresses for the move binary
    ///
    /// Example: alice=0x1234, bob=0x5678
    ///
    /// Note: This will fail if there are duplicates in the Move.toml file remove those first.
    #[clap(long, value_parser = crate::common::utils::parse_map::<String, AccountAddressWrapper>, default_value = "")]
    pub(crate) named_addresses: BTreeMap<String, AccountAddressWrapper>,

    /// Override the standard library version by mainnet/testnet/devnet
    #[clap(long, value_parser)]
    pub override_std: Option<StdVersion>,

    /// Skip pulling the latest git dependencies
    ///
    /// If you don't have a network connection, the compiler may fail due
    /// to no ability to pull git dependencies.  This will allow overriding
    /// this for local development.
    #[clap(long)]
    pub(crate) skip_fetch_latest_git_deps: bool,

    /// Do not complain about unknown attributes in Move code.
    #[clap(long)]
    pub skip_attribute_checks: bool,

    /// Enables dev mode, which uses all dev-addresses and dev-dependencies
    ///
    /// Dev mode allows for changing dependencies and addresses to the preset [dev-addresses] and
    /// [dev-dependencies] fields.  This works both inside and out of tests for using preset values.
    ///
    /// Currently, it also additionally pulls in all test compilation artifacts
    #[clap(long)]
    pub dev: bool,

    /// Do apply extended checks for Aptos (e.g. `#[view]` attribute) also on test code.
    /// NOTE: this behavior will become the default in the future.
    /// See <https://github.com/aptos-labs/aptos-core/issues/10335>
    #[clap(long, env = "APTOS_CHECK_TEST_CODE")]
    pub check_test_code: bool,
}

impl LintPackage {
    fn to_move_options(self) -> MovePackageDir {
        let LintPackage {
            dev,
            package_dir,
            output_dir,
            named_addresses,
            override_std,
            skip_fetch_latest_git_deps,
            language_version,
            skip_attribute_checks,
            check_test_code,
        } = self.clone();
        MovePackageDir {
            dev,
            package_dir,
            output_dir,
            named_addresses,
            override_std,
            skip_fetch_latest_git_deps,
            language_version,
            skip_attribute_checks,
            check_test_code,
            ..MovePackageDir::new()
        }
    }
}

#[async_trait]
impl CliCommand<&'static str> for LintPackage {
    fn command_name(&self) -> &'static str {
        "LintPackage"
    }

    async fn execute(self) -> CliTypedResult<&'static str> {
        let move_options = MovePackageDir {
            compiler_version: Some(CompilerVersion::V2_0),
            ..self.to_move_options()
        };
        let more_experiments = vec![
            Experiment::LINT_CHECKS.to_string(),
            Experiment::SPEC_CHECK.to_string(),
            Experiment::SEQS_IN_BINOPS_CHECK.to_string(),
            Experiment::ACCESS_CHECK.to_string(),
            Experiment::STOP_AFTER_EXTENDED_CHECKS.to_string(),
        ];
        let package_path = move_options.get_package_path()?;
        let included_artifacts = IncludedArtifacts::Sparse;
        let build_options = BuildOptions {
            ..included_artifacts.build_options_with_experiments(
                &move_options,
                more_experiments,
                true,
            )?
        };
        BuiltPackage::build(package_path, build_options)?;
        Ok("succeeded")
    }
}
