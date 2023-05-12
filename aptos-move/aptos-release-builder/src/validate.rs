// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{ExecutionMode, ReleaseConfig};
use anyhow::Result;
use aptos::{
    common::types::CliCommand,
    governance::{ExecuteProposal, SubmitProposal, SubmitVote},
    move_tool::{RunFunction, RunScript},
};
use aptos_api_types::U64;
use aptos_crypto::ed25519::Ed25519PrivateKey;
use aptos_genesis::keys::PrivateIdentity;
use aptos_rest_client::Client;
use aptos_temppath::TempPath;
use aptos_types::account_address::AccountAddress;
use clap::Parser;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};
use url::Url;

pub const FAST_RESOLUTION_TIME: u64 = 30;
pub const DEFAULT_RESOLUTION_TIME: u64 = 43200;

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub endpoint: Url,
    pub root_key_path: PathBuf,
    pub validator_account: AccountAddress,
    pub validator_key: Ed25519PrivateKey,
    pub framework_git_rev: Option<String>,
}

#[derive(Deserialize)]
struct CreateProposalEvent {
    proposal_id: U64,
}

fn aptos_framework_path() -> PathBuf {
    let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    path.pop();
    path.push("framework/aptos-framework");
    path
}

impl NetworkConfig {
    pub fn new_from_dir(endpoint: Url, test_dir: &Path) -> Result<Self> {
        let root_key_path = test_dir.join("mint.key");
        let private_identity_file = test_dir.join("0/private-identity.yaml");
        let private_identity =
            serde_yaml::from_slice::<PrivateIdentity>(&fs::read(private_identity_file)?)?;

        Ok(Self {
            endpoint,
            root_key_path,
            validator_account: private_identity.account_address,
            validator_key: private_identity.account_private_key,
            framework_git_rev: None,
        })
    }

    /// Submit all govenerance proposal script inside script_path to the corresponding rest endpoint.
    ///
    /// For all script, we will:
    /// - Generate a governance proposal and get its proposal id
    /// - Use validator's privkey to vote for this proposal
    /// - Add the proposal to allow list using validator account
    /// - Execute this proposal
    ///
    /// We expect all the scripts here to be single step governance proposal.
    pub async fn submit_and_execute_proposal(&self, script_path: Vec<PathBuf>) -> Result<()> {
        let mut proposals = vec![];
        for path in script_path.iter() {
            let proposal_id = self
                .create_governance_proposal(path.as_path(), false)
                .await?;
            self.vote_proposal(proposal_id).await?;
            proposals.push(proposal_id);
        }

        // Wait for the voting period to pass
        sleep(Duration::from_secs(40));
        for (proposal_id, path) in proposals.iter().zip(script_path.iter()) {
            self.add_proposal_to_allow_list(*proposal_id).await?;
            self.execute_proposal(*proposal_id, path.as_path()).await?;
        }
        Ok(())
    }

    /// Submit all govenerance proposal script inside script_path to the corresponding rest endpoint.
    ///
    /// - We will first submit a governance proposal for the first script (in alphabetical order).
    /// - Validator will vote for this proposal
    ///
    /// Once voting period has passed, we should be able to execute all the scripts in the folder in alphabetical order.
    /// We expect all the scripts here to be multi step governance proposal.
    pub async fn submit_and_execute_multi_step_proposal(
        &self,
        script_path: Vec<PathBuf>,
    ) -> Result<()> {
        let first_script = script_path.first().unwrap();
        let proposal_id = self
            .create_governance_proposal(first_script.as_path(), true)
            .await?;
        self.vote_proposal(proposal_id).await?;
        // Wait for the proposal to resolve.
        sleep(Duration::from_secs(40));
        for path in script_path {
            self.add_proposal_to_allow_list(proposal_id).await?;
            self.execute_proposal(proposal_id, path.as_path()).await?;
        }
        Ok(())
    }

    /// Change the time for a network to resolve governance proposal
    pub async fn set_fast_resolve(&self, resolution_time: u64) -> Result<()> {
        let fast_resolve_script = aptos_temppath::TempPath::new();
        fast_resolve_script.create_as_file()?;
        let mut fas_script_path = fast_resolve_script.path().to_path_buf();
        fas_script_path.set_extension("move");

        std::fs::write(fas_script_path.as_path(), format!(r#"
        script {{
            use aptos_framework::aptos_governance;

            fun main(core_resources: &signer) {{
                let core_signer = aptos_governance::get_signer_testnet_only(core_resources, @0000000000000000000000000000000000000000000000000000000000000001);

                let framework_signer = &core_signer;

                aptos_governance::update_governance_config(framework_signer, 0, 0, {});
            }}
        }}
        "#, resolution_time).as_bytes())?;

        let mut args = vec![
            "",
            "--script-path",
            fas_script_path.as_path().to_str().unwrap(),
            "--sender-account",
            "0xa550c18",
            "--private-key-file",
            self.root_key_path.as_path().to_str().unwrap(),
            "--assume-yes",
            "--encoding",
            "bcs",
            "--url",
            self.endpoint.as_str(),
        ];
        let rev = self.framework_git_rev.clone();
        let framework_path = aptos_framework_path();
        if let Some(rev) = &rev {
            args.push("--framework-git-rev");
            args.push(rev.as_str());
        } else {
            args.push("--framework-local-dir");
            args.push(framework_path.as_os_str().to_str().unwrap());
        };

        RunScript::parse_from(args).execute().await?;
        Ok(())
    }

    pub async fn create_governance_proposal(
        &self,
        script_path: &Path,
        is_multi_step: bool,
    ) -> Result<u64> {
        println!("Creating proposal: {:?}", script_path);

        let address_string = format!("{}", self.validator_account);
        let privkey_string = hex::encode(self.validator_key.to_bytes());

        let mut args = vec![
            "",
            "--pool-address",
            address_string.as_str(),
            "--script-path",
            script_path.to_str().unwrap(),
            "--metadata-url",
            "https://raw.githubusercontent.com/aptos-labs/aptos-core/b4fb9acfc297327c43d030def2b59037c4376611/testsuite/smoke-test/src/upgrade_multi_step_test_metadata.txt",
            "--sender-account",
            address_string.as_str(),
            "--private-key",
            privkey_string.as_str(),
            "--url",
            self.endpoint.as_str(),
            "--assume-yes",
        ];

        if is_multi_step {
            args.push("--is-multi-step");
        }

        let rev_string = self.framework_git_rev.clone();
        let framework_path = aptos_framework_path();
        if let Some(rev) = &rev_string {
            args.push("--framework-git-rev");
            args.push(rev.as_str());
            SubmitProposal::parse_from(args).execute().await?;
        } else {
            args.push("--framework-local-dir");
            args.push(framework_path.as_os_str().to_str().unwrap());
            SubmitProposal::parse_from(args).execute().await?;
        };

        // Get proposal id.
        let event = Client::new(self.endpoint.clone())
            .get_account_events(
                AccountAddress::ONE,
                "0x1::aptos_governance::GovernanceEvents",
                "create_proposal_events",
                None,
                Some(1),
            )
            .await?
            .into_inner()
            .pop()
            .unwrap();

        Ok(*serde_json::from_value::<CreateProposalEvent>(event.data)?
            .proposal_id
            .inner())
    }

    pub async fn vote_proposal(&self, proposal_id: u64) -> Result<()> {
        println!("Voting proposal id {:?}", proposal_id);

        let address_string = format!("{}", self.validator_account);
        let privkey_string = hex::encode(self.validator_key.to_bytes());
        let proposal_id = format!("{}", proposal_id);

        let args = vec![
            "",
            "--pool-addresses",
            address_string.as_str(),
            "--sender-account",
            address_string.as_str(),
            "--private-key",
            privkey_string.as_str(),
            "--assume-yes",
            "--proposal-id",
            proposal_id.as_str(),
            "--yes",
            "--url",
            self.endpoint.as_str(),
        ];

        SubmitVote::parse_from(args).execute().await?;
        Ok(())
    }

    pub async fn mint_to_validator(&self) -> Result<()> {
        let address_args = format!("address:{}", self.validator_account);

        println!("Minting to validator account");
        let args = vec![
            "",
            "--function-id",
            "0x1::aptos_coin::mint",
            "--sender-account",
            "0xa550c18",
            "--args",
            address_args.as_str(),
            "u64:100000000000",
            "--private-key-file",
            self.root_key_path.as_path().to_str().unwrap(),
            "--assume-yes",
            "--encoding",
            "bcs",
            "--url",
            self.endpoint.as_str(),
        ];

        RunFunction::parse_from(args).execute().await?;
        Ok(())
    }

    pub async fn add_proposal_to_allow_list(&self, proposal_id: u64) -> Result<()> {
        let proposal_id = format!("u64:{}", proposal_id);

        let args = vec![
            "",
            "--function-id",
            "0x1::aptos_governance::add_approved_script_hash_script",
            "--sender-account",
            "0xa550c18",
            "--args",
            proposal_id.as_str(),
            "--private-key-file",
            self.root_key_path.as_path().to_str().unwrap(),
            "--assume-yes",
            "--encoding",
            "bcs",
            "--url",
            self.endpoint.as_str(),
        ];
        RunFunction::parse_from(args).execute().await?;
        Ok(())
    }

    pub async fn execute_proposal(&self, proposal_id: u64, script_path: &Path) -> Result<()> {
        println!(
            "Executing: {:?} at proposal id {:?}",
            script_path, proposal_id
        );

        let address_string = format!("{}", self.validator_account);
        let privkey_string = hex::encode(self.validator_key.to_bytes());
        let proposal_id = format!("{}", proposal_id);

        let mut args = vec![
            "",
            "--proposal-id",
            proposal_id.as_str(),
            "--script-path",
            script_path.to_str().unwrap(),
            "--sender-account",
            address_string.as_str(),
            "--private-key",
            privkey_string.as_str(),
            "--assume-yes",
            "--url",
            self.endpoint.as_str(),
            // Use the max gas unit for now. The simulate API sometimes cannot get the right gas estimate for proposals.
            "--max-gas",
            "2000000",
        ];

        let rev = self.framework_git_rev.clone();
        let framework_path = aptos_framework_path();
        if let Some(rev) = &rev {
            args.push("--framework-git-rev");
            args.push(rev.as_str());
        } else {
            args.push("--framework-local-dir");
            args.push(framework_path.as_os_str().to_str().unwrap());
        };

        ExecuteProposal::parse_from(args).execute().await?;
        Ok(())
    }
}

async fn execute_release(
    release_config: ReleaseConfig,
    network_config: NetworkConfig,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    let scripts_path = TempPath::new();
    scripts_path.create_as_dir()?;

    release_config.generate_release_proposal_scripts(
        if let Some(dir) = &output_dir {
            dir.as_path()
        } else {
            scripts_path.path()
        },
    )?;

    for proposal in release_config.proposals {
        let mut proposal_path = scripts_path.path().to_path_buf();
        proposal_path.push("sources");
        proposal_path.push(proposal.name.as_str());

        let mut script_paths: Vec<PathBuf> = std::fs::read_dir(proposal_path.as_path())?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension().map(|s| s == "move").unwrap_or(false) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        script_paths.sort();

        match proposal.execution_mode {
            ExecutionMode::MultiStep => {
                network_config.set_fast_resolve(30).await?;
                network_config
                    .submit_and_execute_multi_step_proposal(script_paths)
                    .await?;

                network_config.set_fast_resolve(43200).await?;
            },
            ExecutionMode::SingleStep => {
                network_config.set_fast_resolve(30).await?;
                // Single step governance proposal;
                network_config
                    .submit_and_execute_proposal(script_paths)
                    .await?;
                network_config.set_fast_resolve(43200).await?;
            },
            ExecutionMode::RootSigner => {
                for entry in script_paths {
                    println!("Executing: {:?}", entry);
                    let mut args = vec![
                        "",
                        "--script-path",
                        entry.as_path().to_str().unwrap(),
                        "--sender-account",
                        "0xa550c18",
                        "--private-key-file",
                        network_config.root_key_path.as_path().to_str().unwrap(),
                        "--assume-yes",
                        "--encoding",
                        "bcs",
                        "--url",
                        network_config.endpoint.as_str(),
                    ];

                    let rev = network_config.framework_git_rev.clone();
                    let framework_path = aptos_framework_path();
                    if let Some(rev) = &rev {
                        args.push("--framework-git-rev");
                        args.push(rev.as_str());
                    } else {
                        args.push("--framework-local-dir");
                        args.push(framework_path.as_os_str().to_str().unwrap());
                    };

                    RunScript::parse_from(args).execute().await?;
                }
            },
        };
    }
    Ok(())
}

pub async fn validate_config(
    release_config: ReleaseConfig,
    network_config: NetworkConfig,
) -> Result<()> {
    validate_config_and_generate_release(release_config, network_config, None).await
}

pub async fn validate_config_and_generate_release(
    release_config: ReleaseConfig,
    network_config: NetworkConfig,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    execute_release(release_config.clone(), network_config.clone(), output_dir).await?;
    release_config.validate_upgrade(network_config.endpoint)
}
