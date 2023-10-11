/// This module defines structs and methods to initialize the gas schedule, which dictates how much
/// it costs to execute Move on the network.
module aptos_framework::gas_schedule {
    use std::error;
    use std::string::String;
    use std::vector;
    use aptos_framework::config_for_next_epoch;

    use aptos_framework::system_addresses;
    use aptos_framework::util::from_bytes;
    use aptos_framework::storage_gas::StorageGasConfig;
    use aptos_framework::storage_gas;

    friend aptos_framework::genesis;

    /// The provided gas schedule bytes are empty or invalid
    const EINVALID_GAS_SCHEDULE: u64 = 1;
    const EINVALID_GAS_FEATURE_VERSION: u64 = 2;

    struct GasEntry has store, copy, drop {
        key: String,
        val: u64,
    }

    struct GasSchedule has key, copy, drop {
        entries: vector<GasEntry>
    }

    struct GasScheduleV2 has key, store, copy, drop {
        feature_version: u64,
        entries: vector<GasEntry>,
    }

    /// Only called during genesis.
    public(friend) fun initialize(aptos_framework: &signer, gas_schedule_blob: vector<u8>) {
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(!vector::is_empty(&gas_schedule_blob), error::invalid_argument(EINVALID_GAS_SCHEDULE));

        // TODO(Gas): check if gas schedule is consistent
        let gas_schedule: GasScheduleV2 = from_bytes(gas_schedule_blob);
        move_to<GasScheduleV2>(aptos_framework, gas_schedule);
    }

    /// This can be called by on-chain governance to update the gas schedule.
    public fun set_gas_schedule(aptos_framework: &signer, gas_schedule_blob: vector<u8>) acquires GasSchedule, GasScheduleV2 {
        system_addresses::assert_aptos_framework(aptos_framework);
        assert!(!vector::is_empty(&gas_schedule_blob), error::invalid_argument(EINVALID_GAS_SCHEDULE));
        let new_gas_schedule: GasScheduleV2 = from_bytes(gas_schedule_blob);
        if (std::features::reconfigure_with_dkg_enabled()) {
            config_for_next_epoch::upsert(aptos_framework, new_gas_schedule);
        } else {
            if (exists<GasScheduleV2>(@aptos_framework)) {
                let gas_schedule = borrow_global_mut<GasScheduleV2>(@aptos_framework);
                assert!(new_gas_schedule.feature_version >= gas_schedule.feature_version,
                    error::invalid_argument(EINVALID_GAS_FEATURE_VERSION));
                // TODO(Gas): check if gas schedule is consistent
                *gas_schedule = new_gas_schedule;
            }
            else {
                if (exists<GasSchedule>(@aptos_framework)) {
                    _ = move_from<GasSchedule>(@aptos_framework);
                };
                // TODO(Gas): check if gas schedule is consistent
                move_to<GasScheduleV2>(aptos_framework, new_gas_schedule);
            };
        }
    }

    public fun on_new_epoch(account: &signer) acquires GasScheduleV2, GasSchedule {
        system_addresses::assert_vm(account);
        if (config_for_next_epoch::does_exist<GasScheduleV2>()) {
            let new_gas_schedule: GasScheduleV2 = config_for_next_epoch::extract<GasScheduleV2>(account);
            if (exists<GasScheduleV2>(@aptos_framework)) {
                let gas_schedule = borrow_global_mut<GasScheduleV2>(@aptos_framework);
                *gas_schedule = new_gas_schedule;
            }
            else {
                if (exists<GasSchedule>(@aptos_framework)) {
                    _ = move_from<GasSchedule>(@aptos_framework);
                };
                move_to<GasScheduleV2>(account, new_gas_schedule);
            };
        }
    }

    public fun set_storage_gas_config(aptos_framework: &signer, config: StorageGasConfig) {
        storage_gas::set_config(aptos_framework, config);
    }
}
