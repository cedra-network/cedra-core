// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::config::{
    config_optimizer::ConfigOptimizer, node_config_loader::NodeType, Error, NodeConfig,
};
use aptos_types::chain_id::ChainId;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

// Useful constants for enabling consensus observer on different node types
const ENABLE_ON_VALIDATORS: bool = true;
const ENABLE_ON_VALIDATOR_FULLNODES: bool = true;
const ENABLE_ON_PUBLIC_FULLNODES: bool = true;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConsensusObserverConfig {
    /// Whether the consensus observer is enabled
    pub observer_enabled: bool,
    /// Whether the consensus observer publisher is enabled
    pub publisher_enabled: bool,

    /// Maximum number of pending network messages
    pub max_network_channel_size: u64,
    /// Maximum number of parallel serialization tasks for message sends
    pub max_parallel_serialization_tasks: usize,
    /// Timeout (in milliseconds) for network RPC requests
    pub network_request_timeout_ms: u64,

    /// Interval (in milliseconds) to garbage collect peer state
    pub garbage_collection_interval_ms: u64,
    /// Maximum number of pending blocks to keep in memory (including payloads, ordered blocks, etc.)
    pub max_num_pending_blocks: u64,
    /// Maximum timeout (in milliseconds) for active subscriptions
    pub max_subscription_timeout_ms: u64,
    /// Maximum timeout (in milliseconds) we'll wait for the synced version to
    /// increase before terminating the active subscription.
    pub max_synced_version_timeout_ms: u64,
    /// Interval (in milliseconds) to check the optimality of the subscribed peers
    pub peer_optimality_check_interval_ms: u64,
    /// Interval (in milliseconds) to check progress of the consensus observer
    pub progress_check_interval_ms: u64,
}

impl Default for ConsensusObserverConfig {
    fn default() -> Self {
        Self {
            observer_enabled: false,
            publisher_enabled: false,
            max_network_channel_size: 1000,
            max_parallel_serialization_tasks: num_cpus::get(), // Default to the number of CPUs
            network_request_timeout_ms: 10_000,                // 10 seconds
            garbage_collection_interval_ms: 60_000,            // 60 seconds
            max_num_pending_blocks: 100,                       // 100 blocks
            max_subscription_timeout_ms: 30_000,               // 30 seconds
            max_synced_version_timeout_ms: 60_000,             // 60 seconds
            peer_optimality_check_interval_ms: 60_000,         // 60 seconds
            progress_check_interval_ms: 5_000,                 // 5 seconds
        }
    }
}

impl ConsensusObserverConfig {
    /// Returns true iff the observer or publisher is enabled
    pub fn is_observer_or_publisher_enabled(&self) -> bool {
        self.observer_enabled || self.publisher_enabled
    }
}

impl ConfigOptimizer for ConsensusObserverConfig {
    fn optimize(
        node_config: &mut NodeConfig,
        local_config_yaml: &Value,
        node_type: NodeType,
        _chain_id: Option<ChainId>,
    ) -> Result<bool, Error> {
        let consensus_observer_config = &mut node_config.consensus_observer;
        let local_observer_config_yaml = &local_config_yaml["consensus_observer"];

        // Check if the observer configs are manually set in the local config.
        // If they are, we don't want to override them.
        let observer_manually_set = !local_observer_config_yaml["observer_enabled"].is_null();
        let publisher_manually_set = !local_observer_config_yaml["publisher_enabled"].is_null();

        // Enable the consensus observer and publisher based on the node type
        let mut modified_config = false;
        match node_type {
            NodeType::Validator => {
                if ENABLE_ON_VALIDATORS && !publisher_manually_set {
                    // Only enable the publisher for validators
                    consensus_observer_config.publisher_enabled = true;
                    modified_config = true;
                }
            },
            NodeType::ValidatorFullnode => {
                if ENABLE_ON_VALIDATOR_FULLNODES
                    && !observer_manually_set
                    && !publisher_manually_set
                {
                    // Enable both the observer and the publisher for VFNs
                    consensus_observer_config.observer_enabled = true;
                    consensus_observer_config.publisher_enabled = true;
                    modified_config = true;
                }
            },
            NodeType::PublicFullnode => {
                if ENABLE_ON_PUBLIC_FULLNODES && !observer_manually_set {
                    // Only enable the observer for PFNs
                    consensus_observer_config.observer_enabled = true;
                    modified_config = true;
                }
            },
        }

        Ok(modified_config)
    }
}
