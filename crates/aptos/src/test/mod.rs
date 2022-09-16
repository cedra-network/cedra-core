// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use crate::account::key_rotation::LookupAddress;
use crate::account::{
    create::{CreateAccount, DEFAULT_FUNDED_COINS},
    fund::FundWithFaucet,
    key_rotation::{RotateKey, RotateSummary},
    list::{ListAccount, ListQuery},
    transfer::{TransferCoins, TransferSummary},
};
use crate::common::init::InitTool;
use crate::common::types::{
    account_address_from_public_key, AccountAddressWrapper, CliError, CliTypedResult,
    EncodingOptions, FaucetOptions, GasOptions, KeyType, MoveManifestAccountWrapper,
    MovePackageDir, OptionalPoolAddressArgs, PrivateKeyInputOptions, PromptOptions,
    PublicKeyInputOptions, RestOptions, RngArgs, SaveFile, TransactionOptions, TransactionSummary,
};
use crate::common::utils::write_to_file;
use crate::move_tool::{
    ArgWithType, CompilePackage, DownloadPackage, IncludedArtifacts, InitPackage, MemberId,
    PublishPackage, RunFunction, TestPackage,
};
use crate::node::{
    AnalyzeMode, AnalyzeValidatorPerformance, InitializeValidator, JoinValidatorSet,
    LeaveValidatorSet, OperatorArgs, OperatorConfigFileArgs, ShowValidatorConfig, ShowValidatorSet,
    ShowValidatorStake, UpdateConsensusKey, UpdateValidatorNetworkAddresses,
    ValidatorConsensusKeyArgs, ValidatorNetworkAddressesArgs,
};
use crate::op::key::{ExtractPeer, GenerateKey, NetworkKeyInputOptions, SaveKey};
use crate::stake::{
    AddStake, IncreaseLockup, InitializeStakeOwner, SetDelegatedVoter, SetOperator, UnlockStake,
    WithdrawStake,
};
use crate::CliCommand;
use aptos_config::config::Peer;
use aptos_crypto::ed25519::Ed25519PublicKey;
use aptos_crypto::{bls12381, ed25519::Ed25519PrivateKey, x25519, PrivateKey};
use aptos_genesis::config::HostAndPort;
use aptos_keygen::KeyGen;
use aptos_logger::warn;
use aptos_rest_client::{aptos_api_types::MoveType, Transaction};
use aptos_sdk::move_types::account_address::AccountAddress;
use aptos_temppath::TempPath;
use aptos_types::on_chain_config::ValidatorSet;
use aptos_types::validator_config::ValidatorConfig;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::{collections::BTreeMap, mem, path::PathBuf, str::FromStr, time::Duration};
use thiserror::private::PathAsDisplay;
use tokio::time::{sleep, Instant};

#[cfg(test)]
mod tests;

pub const INVALID_ACCOUNT: &str = "0xDEADBEEFCAFEBABE";

pub const FIRST_MOVE_FILE: &str = "
module NamedAddress0::store {
    use std::string;
    use aptos_framework::coin::{Self};

    struct CoolCoin has key {}

    public entry fun init(
        account: &signer,
        decimals: u64,
        monitor_supply: bool
    ) {
        let (_, _) = coin::initialize<CoolCoin>(account, string::utf8(b\"CoolCoin\"), string::utf8(b\"COOL\"), decimals, monitor_supply);
        coin::register<CoolCoin>(account);
    }
}";

/// A framework for testing the CLI
pub struct CliTestFramework {
    account_addresses: Vec<AccountAddress>,
    account_keys: Vec<Ed25519PrivateKey>,
    endpoint: Url,
    faucet_endpoint: Url,
    move_dir: Option<PathBuf>,
}

impl CliTestFramework {
    pub fn local_new(num_accounts: usize) -> CliTestFramework {
        let dummy_url = Url::parse("http://localhost").unwrap();
        let mut framework = CliTestFramework {
            account_addresses: Vec::new(),
            account_keys: Vec::new(),
            endpoint: dummy_url.clone(),
            faucet_endpoint: dummy_url,
            move_dir: None,
        };
        let mut keygen = KeyGen::from_seed([0; 32]);
        for _ in 0..num_accounts {
            let key = keygen.generate_ed25519_private_key();
            framework.add_account_to_cli(key);
        }
        framework
    }

    pub async fn new(endpoint: Url, faucet_endpoint: Url, num_accounts: usize) -> CliTestFramework {
        let mut framework = CliTestFramework {
            account_addresses: Vec::new(),
            account_keys: Vec::new(),
            endpoint,
            faucet_endpoint,
            move_dir: None,
        };
        let mut keygen = KeyGen::from_seed([0; 32]);

        for _ in 0..num_accounts {
            framework
                .create_cli_account_from_faucet(keygen.generate_ed25519_private_key(), None)
                .await
                .unwrap();
        }

        framework
    }

    async fn check_account_exists(&self, index: usize) -> bool {
        // Create account if it doesn't exist (and there's a faucet)
        let client = aptos_rest_client::Client::new(self.endpoint.clone());
        let address = self.account_id(index);
        client.get_account(address).await.is_ok()
    }

    pub fn add_account_to_cli(&mut self, private_key: Ed25519PrivateKey) -> usize {
        let address = account_address_from_public_key(&private_key.public_key());
        self.account_addresses.push(address);
        self.account_keys.push(private_key);
        self.account_keys.len() - 1
    }

    pub async fn create_cli_account(
        &mut self,
        private_key: Ed25519PrivateKey,
        sender_index: usize,
    ) -> CliTypedResult<usize> {
        let index = self.add_account_to_cli(private_key);
        if self.check_account_exists(index).await {
            return Err(CliError::UnexpectedError(
                "Account already exists".to_string(),
            ));
        }
        CreateAccount {
            txn_options: self.transaction_options(sender_index, None),
            account: self.account_id(index),
        }
        .execute()
        .await?;

        Ok(index)
    }

    pub async fn create_cli_account_from_faucet(
        &mut self,
        private_key: Ed25519PrivateKey,
        amount: Option<u64>,
    ) -> CliTypedResult<usize> {
        let index = self.add_account_to_cli(private_key);
        if self.check_account_exists(index).await {
            return Err(CliError::UnexpectedError(
                "Account already exists".to_string(),
            ));
        }

        self.fund_account(index, amount).await?;
        warn!("Funded account {:?}", self.account_id(index));
        Ok(index)
    }

    pub async fn fund_account(&self, index: usize, amount: Option<u64>) -> CliTypedResult<String> {
        FundWithFaucet {
            profile_options: Default::default(),
            account: self.account_id(index),
            faucet_options: self.faucet_options(),
            amount: amount.unwrap_or(DEFAULT_FUNDED_COINS),
            rest_options: self.rest_options(),
        }
        .execute()
        .await
    }

    pub async fn lookup_address(
        &self,
        public_key: &Ed25519PublicKey,
    ) -> CliTypedResult<AccountAddress> {
        LookupAddress {
            public_key_options: PublicKeyInputOptions::from_key(public_key),
            rest_options: self.rest_options(),
            encoding_options: Default::default(),
            profile_options: Default::default(),
        }
        .execute()
        .await
    }

    pub async fn rotate_key(
        &mut self,
        index: usize,
        new_private_key: String,
        gas_options: Option<GasOptions>,
    ) -> CliTypedResult<RotateSummary> {
        let response = RotateKey {
            txn_options: TransactionOptions {
                private_key_options: PrivateKeyInputOptions::from_private_key(
                    self.private_key(index),
                )
                .unwrap(),
                sender_account: Some(self.account_id(index)),
                rest_options: self.rest_options(),
                gas_options: gas_options.unwrap_or_default(),
                prompt_options: PromptOptions::no(),
                estimate_max_gas: true,
                ..Default::default()
            },
            new_private_key: Some(new_private_key),
            save_to_profile: None,
            new_private_key_file: None,
        }
        .execute()
        .await?;

        Ok(response)
    }

    pub async fn list_account(&self, index: usize, query: ListQuery) -> CliTypedResult<Vec<Value>> {
        ListAccount {
            rest_options: self.rest_options(),
            profile_options: Default::default(),
            account: Some(self.account_id(index)),
            query,
        }
        .execute()
        .await
    }

    pub async fn transfer_coins(
        &self,
        sender_index: usize,
        receiver_index: usize,
        amount: u64,
        gas_options: Option<GasOptions>,
    ) -> CliTypedResult<TransferSummary> {
        TransferCoins {
            txn_options: self.transaction_options(sender_index, gas_options),
            account: self.account_id(receiver_index),
            amount,
        }
        .execute()
        .await
    }

    pub async fn transfer_invalid_addr(
        &self,
        sender_index: usize,
        amount: u64,
        gas_options: Option<GasOptions>,
    ) -> CliTypedResult<TransferSummary> {
        TransferCoins {
            txn_options: self.transaction_options(sender_index, gas_options),
            account: AccountAddress::from_hex_literal(INVALID_ACCOUNT).unwrap(),
            amount,
        }
        .execute()
        .await
    }

    pub async fn show_validator_config(
        &self,
        pool_index: usize,
    ) -> CliTypedResult<ValidatorConfig> {
        ShowValidatorConfig {
            rest_options: self.rest_options(),
            profile_options: Default::default(),
            operator_args: self.operator_args(Some(pool_index)),
        }
        .execute()
        .await
        .map(|v| (&v).into())
    }

    pub async fn show_validator_set(&self) -> CliTypedResult<ValidatorSet> {
        ShowValidatorSet {
            rest_options: self.rest_options(),
            profile_options: Default::default(),
        }
        .execute()
        .await
        .map(|v| (&v).into())
    }

    pub async fn show_validator_stake(&self, pool_index: usize) -> CliTypedResult<Value> {
        ShowValidatorStake {
            rest_options: self.rest_options(),
            profile_options: Default::default(),
            operator_args: self.operator_args(Some(pool_index)),
        }
        .execute()
        .await
    }

    pub async fn initialize_validator(
        &self,
        index: usize,
        consensus_public_key: bls12381::PublicKey,
        proof_of_possession: bls12381::ProofOfPossession,
        validator_host: HostAndPort,
        validator_network_public_key: x25519::PublicKey,
    ) -> CliTypedResult<TransactionSummary> {
        InitializeValidator {
            txn_options: self.transaction_options(index, None),
            operator_config_file_args: OperatorConfigFileArgs {
                operator_config_file: None,
            },
            validator_consensus_key_args: ValidatorConsensusKeyArgs {
                consensus_public_key: Some(consensus_public_key),
                proof_of_possession: Some(proof_of_possession),
            },
            validator_network_addresses_args: ValidatorNetworkAddressesArgs {
                validator_host: Some(validator_host),
                validator_network_public_key: Some(validator_network_public_key),
                full_node_host: None,
                full_node_network_public_key: None,
            },
        }
        .execute()
        .await
    }

    pub async fn add_stake(&self, index: usize, amount: u64) -> CliTypedResult<TransactionSummary> {
        AddStake {
            txn_options: self.transaction_options(
                index,
                Some(GasOptions {
                    gas_unit_price: Some(1),
                    max_gas: Some(10000),
                }),
            ),
            amount,
        }
        .execute()
        .await
    }

    pub async fn unlock_stake(
        &self,
        index: usize,
        amount: u64,
    ) -> CliTypedResult<TransactionSummary> {
        UnlockStake {
            txn_options: self.transaction_options(index, None),
            amount,
        }
        .execute()
        .await
    }

    pub async fn withdraw_stake(
        &self,
        index: usize,
        amount: u64,
    ) -> CliTypedResult<TransactionSummary> {
        WithdrawStake {
            node_op_options: self.transaction_options(index, None),
            amount,
        }
        .execute()
        .await
    }

    pub async fn increase_lockup(&self, index: usize) -> CliTypedResult<TransactionSummary> {
        IncreaseLockup {
            txn_options: self.transaction_options(index, None),
        }
        .execute()
        .await
    }

    pub async fn join_validator_set(
        &self,
        operator_index: usize,
        pool_index: Option<usize>,
    ) -> CliTypedResult<TransactionSummary> {
        JoinValidatorSet {
            txn_options: self.transaction_options(operator_index, None),
            operator_args: self.operator_args(pool_index),
        }
        .execute()
        .await
    }

    pub async fn leave_validator_set(
        &self,
        operator_index: usize,
        pool_index: Option<usize>,
    ) -> CliTypedResult<TransactionSummary> {
        LeaveValidatorSet {
            txn_options: self.transaction_options(operator_index, None),
            operator_args: self.operator_args(pool_index),
        }
        .execute()
        .await
    }

    pub async fn update_validator_network_addresses(
        &self,
        operator_index: usize,
        pool_index: Option<usize>,
        validator_host: HostAndPort,
        validator_network_public_key: x25519::PublicKey,
    ) -> CliTypedResult<TransactionSummary> {
        UpdateValidatorNetworkAddresses {
            txn_options: self.transaction_options(operator_index, None),
            operator_args: self.operator_args(pool_index),
            operator_config_file_args: OperatorConfigFileArgs {
                operator_config_file: None,
            },
            validator_network_addresses_args: ValidatorNetworkAddressesArgs {
                validator_host: Some(validator_host),
                validator_network_public_key: Some(validator_network_public_key),
                full_node_host: None,
                full_node_network_public_key: None,
            },
        }
        .execute()
        .await
    }

    pub async fn analyze_validator_performance(
        &self,
        start_epoch: Option<i64>,
        end_epoch: Option<i64>,
    ) -> CliTypedResult<()> {
        AnalyzeValidatorPerformance {
            start_epoch: start_epoch.unwrap_or(-2),
            end_epoch,
            rest_options: self.rest_options(),
            profile_options: Default::default(),
            analyze_mode: AnalyzeMode::All,
        }
        .execute()
        .await
    }

    pub async fn update_consensus_key(
        &self,
        operator_index: usize,
        pool_index: Option<usize>,
        consensus_public_key: bls12381::PublicKey,
        proof_of_possession: bls12381::ProofOfPossession,
    ) -> CliTypedResult<TransactionSummary> {
        UpdateConsensusKey {
            txn_options: self.transaction_options(operator_index, None),
            operator_args: self.operator_args(pool_index),
            operator_config_file_args: OperatorConfigFileArgs {
                operator_config_file: None,
            },
            validator_consensus_key_args: ValidatorConsensusKeyArgs {
                consensus_public_key: Some(consensus_public_key),
                proof_of_possession: Some(proof_of_possession),
            },
        }
        .execute()
        .await
    }

    pub async fn init(&self, private_key: &Ed25519PrivateKey) -> CliTypedResult<()> {
        InitTool {
            rest_url: Some(self.endpoint.clone()),
            faucet_url: Some(self.faucet_endpoint.clone()),
            rng_args: RngArgs::from_seed([0; 32]),
            private_key_options: PrivateKeyInputOptions::from_private_key(private_key)?,
            profile_options: Default::default(),
            prompt_options: PromptOptions::yes(),
            encoding_options: EncodingOptions::default(),
            skip_faucet: false,
        }
        .execute()
        .await
    }

    pub async fn initialize_stake_owner(
        &self,
        owner_index: usize,
        initial_stake_amount: u64,
        voter_index: Option<usize>,
        operator_index: Option<usize>,
    ) -> CliTypedResult<TransactionSummary> {
        InitializeStakeOwner {
            txn_options: self.transaction_options(
                owner_index,
                Some(GasOptions {
                    gas_unit_price: Some(1),
                    max_gas: Some(10000),
                }),
            ),
            initial_stake_amount,
            operator_address: operator_index.map(|idx| self.account_id(idx)),
            voter_address: voter_index.map(|idx| self.account_id(idx)),
        }
        .execute()
        .await
    }

    pub async fn set_operator(
        &self,
        owner_index: usize,
        operator_index: usize,
    ) -> CliTypedResult<TransactionSummary> {
        SetOperator {
            txn_options: self.transaction_options(owner_index, None),
            operator_address: self.account_id(operator_index),
        }
        .execute()
        .await
    }

    pub async fn set_delegated_voter(
        &self,
        owner_index: usize,
        voter_index: usize,
    ) -> CliTypedResult<TransactionSummary> {
        SetDelegatedVoter {
            txn_options: self.transaction_options(owner_index, None),
            voter_address: self.account_id(voter_index),
        }
        .execute()
        .await
    }

    /// Wait for an account to exist
    pub async fn wait_for_account(&self, index: usize) -> CliTypedResult<Vec<Value>> {
        let mut result = self.list_account(index, ListQuery::Balance).await;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            match result {
                Ok(_) => return result,
                _ => {
                    sleep(Duration::from_millis(500)).await;
                    result = self.list_account(index, ListQuery::Balance).await;
                }
            };
        }

        result
    }

    pub async fn account_balance_now(&self, index: usize) -> CliTypedResult<u64> {
        let result = self.list_account(index, ListQuery::Balance).await?;
        Ok(json_account_to_balance(result.first().unwrap()))
    }

    pub async fn assert_account_balance_now(&self, index: usize, expected: u64) {
        let result = self.list_account(index, ListQuery::Balance).await;
        assert!(
            result.is_ok(),
            "Account {} not yet created, {}, last 10 transactions: {}",
            self.account_id(index),
            result.unwrap_err(),
            self.last_n_transactions_details(10).await
        );
        let accounts = result.unwrap();
        let account = accounts.first().unwrap();
        let coin = json_account_to_balance(account);
        assert_eq!(
            coin,
            expected,
            "Account {} with state: {:?}, last 10 transactions: {}",
            self.account_id(index),
            account,
            self.last_n_transactions_details(10).await
        );
    }

    async fn last_n_transactions_details(&self, count: u16) -> String {
        let result = aptos_rest_client::Client::new(self.endpoint.clone())
            .get_transactions(None, Some(count))
            .await;
        if let Err(e) = result {
            return format!("Err({:?})", e);
        }
        let lines = result
            .unwrap()
            .inner()
            .iter()
            .map(|t| {
                if let Transaction::UserTransaction(u) = t {
                    format!(
                        " * [{}] {}: sender={}, payload={:?}",
                        t.version().unwrap_or(0),
                        t.vm_status(),
                        u.request.sender,
                        u.request.payload
                    )
                } else {
                    format!(
                        " * [{}] {}: {}",
                        t.version().unwrap_or(0),
                        t.vm_status(),
                        t.type_str()
                    )
                }
            })
            .collect::<Vec<_>>();
        format!("\n{}\n", lines.join("\n"))
    }

    pub async fn generate_x25519_key(
        &self,
        output_file: PathBuf,
        seed: [u8; 32],
    ) -> CliTypedResult<HashMap<&'static str, PathBuf>> {
        GenerateKey {
            key_type: KeyType::X25519,
            rng_args: RngArgs::from_seed(seed),
            save_params: SaveKey {
                file_options: SaveFile {
                    output_file,
                    prompt_options: PromptOptions::yes(),
                },
                encoding_options: Default::default(),
            },
        }
        .execute()
        .await
    }

    pub async fn extract_peer(
        &self,
        host: HostAndPort,
        private_key_file: PathBuf,
        output_file: PathBuf,
    ) -> CliTypedResult<HashMap<AccountAddress, Peer>> {
        ExtractPeer {
            host,
            network_key_input_options: NetworkKeyInputOptions::from_private_key_file(
                private_key_file,
            ),
            output_file_options: SaveFile {
                output_file,
                prompt_options: PromptOptions::yes(),
            },
            encoding_options: Default::default(),
        }
        .execute()
        .await
    }

    pub fn init_move_dir(&mut self) {
        let move_dir = TempPath::new();
        move_dir
            .create_as_dir()
            .expect("Expected to be able to create move temp dir");
        self.move_dir = Some(move_dir.path().to_path_buf());
    }

    pub fn add_move_files(&self) {
        let move_dir = self.move_dir();
        let sources_dir = move_dir.join("sources");

        let hello_blockchain_contents = include_str!(
            "../../../../aptos-move/move-examples/hello_blockchain/sources/hello_blockchain.move"
        );
        let source_path = sources_dir.join("hello_blockchain.move");
        write_to_file(
            source_path.as_path(),
            &source_path.as_display().to_string(),
            hello_blockchain_contents.as_bytes(),
        )
        .unwrap();

        let hello_blockchain_test_contents = include_str!("../../../../aptos-move/move-examples/hello_blockchain/sources/hello_blockchain_test.move");
        let test_path = sources_dir.join("hello_blockchain_test.move");
        write_to_file(
            test_path.as_path(),
            &test_path.as_display().to_string(),
            hello_blockchain_test_contents.as_bytes(),
        )
        .unwrap();
    }

    pub fn move_dir(&self) -> PathBuf {
        assert!(self.move_dir.is_some(), "Must have initialized the temp move directory with `CliTestFramework::init_move_dir()` first");
        self.move_dir.as_ref().cloned().unwrap()
    }

    pub async fn init_package(
        &self,
        name: String,
        account_strs: BTreeMap<&str, &str>,
        framework_dir: Option<PathBuf>,
    ) -> CliTypedResult<()> {
        InitPackage {
            name,
            package_dir: Some(self.move_dir()),
            named_addresses: Self::move_manifest_named_addresses(account_strs),
            prompt_options: PromptOptions {
                assume_yes: false,
                assume_no: true,
            },
            for_test_framework: framework_dir,
        }
        .execute()
        .await
    }

    pub async fn compile_package(
        &self,
        account_strs: BTreeMap<&str, &str>,
        included_artifacts: Option<IncludedArtifacts>,
    ) -> CliTypedResult<Vec<String>> {
        CompilePackage {
            move_options: self.move_options(account_strs),
            save_metadata: false,
            included_artifacts: included_artifacts.unwrap_or(IncludedArtifacts::Sparse),
        }
        .execute()
        .await
    }

    pub async fn test_package(
        &self,
        account_strs: BTreeMap<&str, &str>,
        filter: Option<&str>,
    ) -> CliTypedResult<&'static str> {
        TestPackage {
            instruction_execution_bound: 100_000,
            move_options: self.move_options(account_strs),
            filter: filter.map(|str| str.to_string()),
        }
        .execute()
        .await
    }

    pub async fn publish_package(
        &self,
        index: usize,
        gas_options: Option<GasOptions>,
        account_strs: BTreeMap<&str, &str>,
        legacy_flow: bool,
        included_artifacts: Option<IncludedArtifacts>,
    ) -> CliTypedResult<TransactionSummary> {
        PublishPackage {
            move_options: self.move_options(account_strs),
            txn_options: self.transaction_options(index, gas_options),
            legacy_flow,
            override_size_check: false,
            included_artifacts: included_artifacts.unwrap_or(IncludedArtifacts::All),
        }
        .execute()
        .await
    }

    pub async fn download_package(
        &self,
        index: usize,
        package: String,
        output_dir: PathBuf,
    ) -> CliTypedResult<&'static str> {
        DownloadPackage {
            rest_options: self.rest_options(),
            profile_options: Default::default(),
            account: self.account_id(index),
            package,
            output_dir: Some(output_dir),
        }
        .execute()
        .await
    }

    pub async fn run_function(
        &self,
        index: usize,
        gas_options: Option<GasOptions>,
        function_id: MemberId,
        args: Vec<&str>,
        type_args: Vec<&str>,
    ) -> CliTypedResult<TransactionSummary> {
        let mut parsed_args = Vec::new();
        for arg in args {
            parsed_args.push(
                ArgWithType::from_str(arg)
                    .map_err(|err| CliError::UnexpectedError(err.to_string()))?,
            )
        }

        let mut parsed_type_args = Vec::new();
        for arg in type_args {
            parsed_type_args.push(
                MoveType::from_str(arg)
                    .map_err(|err| CliError::UnexpectedError(err.to_string()))?,
            )
        }

        RunFunction {
            txn_options: self.transaction_options(index, gas_options),
            function_id,
            args: parsed_args,
            type_args: parsed_type_args,
        }
        .execute()
        .await
    }

    pub fn move_options(&self, account_strs: BTreeMap<&str, &str>) -> MovePackageDir {
        MovePackageDir {
            package_dir: Some(self.move_dir()),
            output_dir: None,
            named_addresses: Self::named_addresses(account_strs),
        }
    }

    pub fn move_manifest_named_addresses(
        account_strs: BTreeMap<&str, &str>,
    ) -> BTreeMap<String, MoveManifestAccountWrapper> {
        account_strs
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    MoveManifestAccountWrapper::from_str(value).unwrap(),
                )
            })
            .collect()
    }

    pub fn named_addresses(
        account_strs: BTreeMap<&str, &str>,
    ) -> BTreeMap<String, AccountAddressWrapper> {
        account_strs
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    AccountAddressWrapper::from_str(value).unwrap(),
                )
            })
            .collect()
    }

    pub fn rest_options(&self) -> RestOptions {
        RestOptions::new(Some(self.endpoint.clone()))
    }

    pub fn faucet_options(&self) -> FaucetOptions {
        FaucetOptions::new(Some(self.faucet_endpoint.clone()))
    }

    fn transaction_options(
        &self,
        index: usize,
        gas_options: Option<GasOptions>,
    ) -> TransactionOptions {
        TransactionOptions {
            private_key_options: PrivateKeyInputOptions::from_private_key(self.private_key(index))
                .unwrap(),
            sender_account: Some(self.account_id(index)),
            rest_options: self.rest_options(),
            gas_options: gas_options.unwrap_or_default(),
            prompt_options: PromptOptions::yes(),
            estimate_max_gas: true,
            ..Default::default()
        }
    }

    fn operator_args(&self, pool_index: Option<usize>) -> OperatorArgs {
        OperatorArgs {
            pool_address_args: OptionalPoolAddressArgs {
                pool_address: pool_index.map(|idx| self.account_id(idx)),
            },
        }
    }

    pub fn private_key(&self, index: usize) -> &Ed25519PrivateKey {
        self.account_keys.get(index).unwrap()
    }

    pub fn set_private_key(
        &mut self,
        index: usize,
        new_key: Ed25519PrivateKey,
    ) -> Ed25519PrivateKey {
        // Insert the new private key into the test framework, returning the old one
        mem::replace(&mut self.account_keys[index], new_key)
    }

    pub fn account_id(&self, index: usize) -> AccountAddress {
        *self.account_addresses.get(index).unwrap()
    }
}

// ValidatorConfig/ValidatorSet doesn't match Move ValidatorSet struct,
// and json is serialized with different types from both, so hardcoding deserialization.

fn json_account_to_balance(value: &Value) -> u64 {
    u64::from_str(
        value
            .as_object()
            .unwrap()
            .get("coin")
            .unwrap()
            .as_object()
            .unwrap()
            .get("value")
            .unwrap()
            .as_str()
            .unwrap(),
    )
    .unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndividualValidatorPerformance {
    successful_proposals: String,
    failed_proposals: String,
}

impl IndividualValidatorPerformance {
    pub fn successful_proposals(&self) -> u32 {
        self.successful_proposals.parse().unwrap()
    }

    pub fn failed_proposals(&self) -> u32 {
        self.failed_proposals.parse().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidatorPerformance {
    pub validators: Vec<IndividualValidatorPerformance>,
}
