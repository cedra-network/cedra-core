// Copyright (c) Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, time::Instant};
use rand::Rng;
use rand::seq::SliceRandom;
use aptos_block_partitioner::test_utils::{create_signed_p2p_transaction, generate_test_account, TestAccount};
use aptos_types::transaction::analyzed_transaction::AnalyzedTransaction;
use rand_distr::{Distribution, LogNormal, Normal};
use aptos_logger::info;
use rayon::prelude::*;
use aptos_transaction_orderer::common::PTransaction;

#[derive(Debug)]
pub struct ClusteredTxnsGenConfig {
    pub num_clusters: usize,
    pub mean_txns_per_user: usize,
    pub num_resource_addresses_per_cluster: usize,
    pub cluster_size_relative_std_dev: f64,
    pub txns_per_user_relative_std_dev: f64,
    pub fraction_of_external_txns: f64,
    pub print_debug_stats: bool,
    pub total_user_accounts: usize,
}

pub struct ClusteredTxnsGenerator {
    num_clusters: usize,
    mean_txns_per_user: usize,
    num_resource_addresses_per_cluster: usize,
    cluster_size_relative_std_dev: f64,
    txns_per_user_relative_std_dev: f64,
    fraction_of_external_txns: f64,
    all_user_accounts: Vec<TestAccount>,
    cluster_resource_addresses: Vec<Vec<TestAccount>>,
    print_debug_stats: bool,
    slow_gen_accounts: bool,
}

impl ClusteredTxnsGenerator {
    pub fn new(
        num_clusters: usize,
        total_user_accounts: usize,
        num_resource_addresses_per_cluster: usize,
        mean_txns_per_user: usize,
        cluster_size_relative_std_dev: f64,
        txns_per_user_relative_std_dev: f64,
        fraction_of_external_txns: f64,
        print_debug_stats: bool,
        gen_accounts: bool,
        slow_gen_accounts: bool,
    ) -> Self {
        let all_user_accounts = if gen_accounts {
            (0..total_user_accounts)
                .map(|_| generate_test_account())
                .collect()
        } else {
            vec![]
        };

        let cluster_resource_addresses = if gen_accounts {
            (0..num_clusters)
                .map(|_| {
                    (0..num_resource_addresses_per_cluster)
                        .map(|_| generate_test_account())
                        .collect()
                }).collect()
        } else {
            vec![]
        };

        Self {
            num_clusters,
            mean_txns_per_user,
            num_resource_addresses_per_cluster,
            cluster_size_relative_std_dev,
            txns_per_user_relative_std_dev,
            fraction_of_external_txns,
            all_user_accounts,
            cluster_resource_addresses,
            print_debug_stats,
            slow_gen_accounts,
        }
    }

    fn generate_normal_distribution(num_buckets: usize, total_samples: usize, bucket_size_std_dev: f64) -> Vec<usize> {
        let mean_bucket_size = total_samples as f64 / num_buckets as f64;
        //info!("num_buckets: {}, total_samples: {}, bucket_size_std_dev: {}; mean_bucket_size: {}",
          //       num_buckets, total_samples, bucket_size_std_dev, mean_bucket_size);
        let normal = Normal::new(mean_bucket_size, bucket_size_std_dev).unwrap();
        let mut rng = rand::thread_rng();
        let mut cluster_sizes: Vec<usize> = (0..num_buckets)
            .map(|_| {
                let mut size;
                loop {
                    size = normal.sample(&mut rng).round() as isize;
                   // info!("size: {}", size);
                    if size >= 0 {
                        size = size.min((mean_bucket_size * 10.0) as isize);
                        break;
                    }
                }
                size as usize
            })
            .collect();

        // Adjust the sizes to ensure the total number of users matches num_users
        let total_size: usize = cluster_sizes.iter().sum();
        let mut diff = total_size as isize - total_samples as isize;

        while diff != 0 {
            for size in cluster_sizes.iter_mut() {
                if diff == 0 {
                    break;
                }
                if diff > 0 && *size > 0 {
                    *size -= 1;
                    diff -= 1;
                } else if diff < 0 {
                    *size += 1;
                    diff += 1;
                }
            }
        }

        cluster_sizes
    }

    fn generate_log_normal_distribution(num_buckets: usize, total_samples: usize, bucket_size_std_dev: f64) -> Vec<usize> {
        let mean_bucket_size: f64 = total_samples as f64 / num_buckets as f64;
        //info!("num_buckets: {}, total_samples: {}, bucket_size_std_dev: {}; mean_bucket_size: {}",
          //       num_buckets, total_samples, bucket_size_std_dev, mean_bucket_size);
        let log_normal = LogNormal::new(mean_bucket_size.ln(), bucket_size_std_dev).unwrap();
        let mut rng = rand::thread_rng();
        let mut cluster_sizes: Vec<usize> = (0..num_buckets)
            .map(|_i| {
                // Note: log_normal.sample() returns a value in the range (0, +inf)
                let size= log_normal.sample(&mut rng).round();
            //    info!("i: {}, size: {}", i, size);
                assert!(size >= 0.0);
                size.min(mean_bucket_size * 10.0) as usize
            })
            .collect();

        // Adjust the sizes to ensure the total number of users matches num_users
        let total_size: usize = cluster_sizes.iter().sum();
        let mut diff = total_size as isize - total_samples as isize;

        while diff != 0 {
            for size in cluster_sizes.iter_mut() {
                if diff == 0 {
                    break;
                }
                if diff > 0 && *size > 1 {
                    *size -= 1;
                    diff -= 1;
                } else if diff < 0 {
                    *size += 1;
                    diff += 1;
                }
            }
        }

        cluster_sizes
    }

    pub fn generate_txn_indices(&self, num_txns: usize) -> Vec<(usize, (usize, usize))> {
        let mut indices = vec![];

        // distribute the user accounts among the clusters
        let num_users = num_txns / self.mean_txns_per_user;
        let mean_users_per_cluster = num_users / self.num_clusters;
        let cluster_size_std_dev = self.cluster_size_relative_std_dev * mean_users_per_cluster as f64;
        let cluster_sizes = Self::generate_normal_distribution(self.num_clusters, num_users, cluster_size_std_dev);
        assert_eq!(cluster_sizes.iter().sum::<usize>(), num_users);
        //info!("cluster_sizes: {:?}", cluster_sizes);

        // generate distribution on number of txns per user
        let txns_per_user_std_dev = self.txns_per_user_relative_std_dev * self.mean_txns_per_user as f64;
        let txns_per_user = Self::generate_log_normal_distribution(num_users, num_txns, txns_per_user_std_dev);
        //info!("txns_per_user: {:?}", txns_per_user);

        // user accounts : 0 --> num_users
        // cluster 0 : user[0] --> user[cluster_sizes[0] - 1]; cluster 1 : user[cluster_sizes[0]] --> user[cluster_sizes[0] + cluster_sizes[1] - 1]; ...
        let mut debug_cluster_to_inactive_users = vec![0; self.num_clusters];
        let mut debug_cluster_to_num_txns = vec![0; self.num_clusters];
        let mut debug_cluster_to_external_txns = vec![0; self.num_clusters];
        let mut user_idx = 0;
        for (cluster_idx, cluster_size) in cluster_sizes.iter().enumerate() {
            let user_idx_end = user_idx + cluster_size;
            //info!("cluster_idx: {}, cluster_size: {}, user_idx: {}, user_idx_end: {}", cluster_idx, cluster_size, user_idx, user_idx_end);
            while user_idx < user_idx_end {
                let num_txns_for_user = txns_per_user[user_idx];
                debug_cluster_to_num_txns[cluster_idx] += num_txns_for_user;
                if num_txns_for_user == 0 {
                    debug_cluster_to_inactive_users[cluster_idx] += 1;
                }
                //info!("user_idx: {}, num_txns_for_user: {}", user_idx, num_txns_for_user);
                for _ in 0..num_txns_for_user {
                    let is_external = rand::thread_rng().gen_bool(self.fraction_of_external_txns);
                    let (recvr_cluster, recvr_resource_idx) = if is_external {
                        debug_cluster_to_external_txns[cluster_idx] += 1;
                        let mut external_cluster;
                        loop {
                            external_cluster = rand::thread_rng().gen_range(0..self.num_clusters);
                            if external_cluster != cluster_idx {
                                break;
                            }
                        }
                        let recvr_resource_idx = rand::thread_rng().gen_range(0..self.num_resource_addresses_per_cluster);
                        (external_cluster, recvr_resource_idx)
                    } else {
                        let recvr_resource_idx = rand::thread_rng().gen_range(0..self.num_resource_addresses_per_cluster);
                        (cluster_idx, recvr_resource_idx)
                    };
                    indices.push((user_idx, (recvr_cluster, recvr_resource_idx)));
                }
                user_idx += 1;
            }
        }
        assert_eq!(indices.len(), num_txns);

        if self.print_debug_stats {
            info!("cluster_sizes: {:?}", cluster_sizes);
            for (cluster_idx, cluster_size) in cluster_sizes.iter().enumerate() {
                info!("cluster_id: {}; user_count: {}; txn_count: {}; external_txns: {}; inactive_users_count: {};",
                         cluster_idx, cluster_size, debug_cluster_to_num_txns[cluster_idx], debug_cluster_to_external_txns[cluster_idx], debug_cluster_to_inactive_users[cluster_idx]);
            }
            info!("Total external txns: {}; total inactive user: {}",
                     debug_cluster_to_external_txns.iter().sum::<usize>(),
                     debug_cluster_to_inactive_users.iter().sum::<usize>()
            );
        }
        indices.shuffle(&mut rand::thread_rng());
        indices
    }

    pub fn generate(&self, num_txns: usize) -> Vec<AnalyzedTransaction> {
        assert!(self.all_user_accounts.len() * self.mean_txns_per_user >= 2 * num_txns);
        info!("Generating Clustered groups of txns =================================");

        let start_time = Instant::now();
        let txn_indices = self.generate_txn_indices(num_txns);
        //let txn_indices = [(0, (0, 0)), (1, (0, 1)), (2, (0, 2)), (3, (0, 2)), (3, (0, 3)), (5, (0, 4)), (6, (0, 5)), (7, (0, 6))]; // preserves sender order
        //let txn_indices = [(0, (0, 0)), (1, (0, 0)), (2, (0, 0)), (3, (0, 0)), (4, (0, 1)), (5, (0, 1)), (6, (0, 1)), (7, (0, 1))]; // simple re-order with window_size == 2
        //let txn_indices = [(0, (0, 0)), (1, (0, 1)), (2, (0, 2)), (3, (0, 2)), (4, (0, 2)), (5, (0, 2)), (6, (0, 3)), (7, (0, 4))]; // complex re-order with window_size == 2
        //let txn_indices = [(13, (1, 0)), (4, (1, 1)), (7, (0, 0)), (10, (0, 0)), (3, (0, 1)), (0, (0, 1)), (12, (1, 1)), (1, (0, 1)), (5, (0, 1)), (11, (0, 0)), (14, (0, 0)), (13, (1, 0)), (10, (1, 0)), (9, (1, 0)), (13, (1, 1)), (12, (1, 1)), (12, (1, 1)), (13, (0, 0)), (3, (0, 0)), (3, (1, 1)), (10, (1, 0)), (9, (1, 1)), (8, (1, 1)), (9, (1, 1)), (2, (0, 0)), (3, (0, 1)), (3, (0, 0)), (6, (0, 1)), (3, (0, 1)), (2, (1, 1))];
        //println!("txn_indices: {:?}", txn_indices);
        let duration = start_time.elapsed();
        info!("Time taken to generate txn_indices: {:?}", duration);

        let start_time = Instant::now();

        let mut txns: Vec<AnalyzedTransaction> =
            if !self.slow_gen_accounts {
                let mut by_sender = HashMap::new();
                for (sender_idx, (recvr_cluster, recvr_resource_idx)) in txn_indices {
                    by_sender.entry(sender_idx).or_insert(Vec::new()).push((recvr_cluster, recvr_resource_idx));
                }
                by_sender.par_iter().map(|(sender_idx, recvs)| {
                    let receivers = recvs.iter().map(|(recvr_cluster, recvr_resource_idx)| &self.cluster_resource_addresses[*recvr_cluster][*recvr_resource_idx]).collect::<Vec<_>>();
                    let sender = &self.all_user_accounts[*sender_idx];
                    create_signed_p2p_transaction(sender, receivers)
                }).flatten().collect()
            } else {
                txn_indices.iter().map(|(sender_idx, (recvr_cluster, recvr_resource_idx))| {
                    let receivers = vec![&self.cluster_resource_addresses[*recvr_cluster][*recvr_resource_idx]];
                    let sender = &self.all_user_accounts[*sender_idx];
                    create_signed_p2p_transaction(sender, receivers)
                }).flatten().collect()
            };

        for (idx, txn) in txns.iter_mut().enumerate() {
            txn.id = idx;
        }

        let duration = start_time.elapsed();
        info!("Time taken to create p2p txns: {:?}", duration);

        info!("Generated {} txns =================================", txns.len());
        txns
    }
}