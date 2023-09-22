spec aptos_framework::staking_proxy {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    /// Aborts if conditions of SetStakePoolOperator are not met
    spec set_operator(owner: &signer, old_operator: address, new_operator: address) {
        pragma verify_duration_estimate = 360; // TODO: set because of timeout (property proved)
        // TODO: Can't verify `set_vesting_contract_operator` and `set_staking_contract_operator`
        pragma aborts_if_is_partial;
        include SetStakePoolOperator;
        include SetStakingContractOperator;
    }

    /// Aborts if conditions of SetStackingContractVoter and SetStackPoolVoterAbortsIf are not met
    spec set_voter(owner: &signer, operator: address, new_voter: address) {
        // TODO: Can't verify `set_vesting_contract_voter`
        pragma aborts_if_is_partial;
        include SetStakingContractVoter;
        include SetStakePoolVoterAbortsIf;
    }

    spec set_vesting_contract_operator(owner: &signer, old_operator: address, new_operator: address) {
        // TODO: Can't verify `update_voter` in while loop.
        pragma aborts_if_is_partial;

        let owner_address = signer::address_of(owner);
        let vesting_contracts = global<vesting::AdminStore>(owner_address).vesting_contracts;
        // let post post_vesting_contracts = global<vesting::AdminStore>(owner_address).vesting_contracts;

        // aborts_if exists<vesting::AdminStore>(owner_address) && len(vesting_contracts) != 0 &&
        //     (exists i in 0..len(vesting_contracts): !exists<vesting::VestingContract>(vesting_contracts[i]));

        // aborts_if exists<vesting::AdminStore>(owner_address) && len(vesting_contracts) != 0 &&
        //     (exists i in 0..len(vesting_contracts): global<vesting::VestingContract>(vesting_contracts[i]).staking.operator == old_operator
        //     && owner_address != global<vesting::VestingContract>(vesting_contracts[i]).admin);


        // ensures exists<vesting::AdminStore>(owner_address) && len(vesting_contracts) != 0 ==>
        //     (exists i in 0..len(vesting_contracts): old(global<vesting::VestingContract>(vesting_contracts[i])).staking.operator == old_operator
        //         && global<vesting::VestingContract>(vesting_contracts[i]).staking.operator == new_operator);

        // ensures exists<vesting::AdminStore>(owner_address) && len(vesting_contracts) != 0 ==>
        //     (exists i in 0..len(vesting_contracts): (old(global<vesting::VestingContract>(vesting_contracts[i])).staking.operator == old_operator
        //         ==> global<vesting::VestingContract>(vesting_contracts[i]).staking.operator == new_operator));
    }

    spec set_staking_contract_operator(owner: &signer, old_operator: address, new_operator: address) {
        // TODO: Verify timeout and can't verify `staking_contract::switch_operator`.
        pragma aborts_if_is_partial;
        pragma verify_duration_estimate = 120; // TODO: set because of timeout (property proved)

        include SetStakingContractOperator;
    }

    spec schema SetStakingContractOperator {
        use aptos_std::simple_map;
        use aptos_framework::staking_contract::{Store};
        use aptos_framework::coin;

        owner: signer; 
        old_operator: address; 
        new_operator: address;

        let owner_address = signer::address_of(owner);
        let store = global<Store>(owner_address);
        let staking_contract_exists = exists<Store>(owner_address) && simple_map::spec_contains_key(store.staking_contracts, old_operator);
        aborts_if staking_contract_exists && simple_map::spec_contains_key(store.staking_contracts, new_operator);
        
        let post post_store = global<Store>(owner_address);
        ensures staking_contract_exists ==> !simple_map::spec_contains_key(post_store.staking_contracts, old_operator);

        let staking_contract = simple_map::spec_get(store.staking_contracts, old_operator);
        let stake_pool = global<stake::StakePool>(staking_contract.pool_address);
        let active = coin::value(stake_pool.active);
        let pending_active = coin::value(stake_pool.pending_active);
        let total_active_stake = active + pending_active;
        let accumulated_rewards = total_active_stake - staking_contract.principal;
        let commission_amount = accumulated_rewards * staking_contract.commission_percentage / 100;
        aborts_if staking_contract_exists && !exists<stake::StakePool>(staking_contract.pool_address);
        ensures staking_contract_exists ==> 
            simple_map::spec_get(post_store.staking_contracts, new_operator).principal == total_active_stake - commission_amount;
        
        let pool_address = staking_contract.owner_cap.pool_address;
        let current_commission_percentage = staking_contract.commission_percentage;
        aborts_if staking_contract_exists && commission_amount != 0 && !exists<stake::StakePool>(pool_address);
        ensures staking_contract_exists && commission_amount != 0 ==> 
            global<stake::StakePool>(pool_address).operator_address == new_operator
            && simple_map::spec_get(post_store.staking_contracts, new_operator).commission_percentage == current_commission_percentage;
        
        ensures staking_contract_exists ==> simple_map::spec_contains_key(post_store.staking_contracts, new_operator);
    }

    spec set_vesting_contract_voter(owner: &signer, operator: address, new_voter: address) {
        // TODO: Can't verify `update_voter` in while loop.
        pragma aborts_if_is_partial;
    }

    /// Aborts if stake_pool is exists and when OwnerCapability or stake_pool_exists
    /// One of them are not exists
    spec set_stake_pool_operator(owner: &signer, new_operator: address) {
        include SetStakePoolOperator;
    }

    spec schema SetStakePoolOperator {
        owner: &signer;
        new_operator: address;

        let owner_address = signer::address_of(owner);
        let ownership_cap = borrow_global<stake::OwnerCapability>(owner_address);
        let pool_address = ownership_cap.pool_address;
        aborts_if stake::stake_pool_exists(owner_address) && !(exists<stake::OwnerCapability>(owner_address) && stake::stake_pool_exists(pool_address));
        ensures stake::stake_pool_exists(owner_address) ==> global<stake::StakePool>(pool_address).operator_address == new_operator;
    }

    spec set_staking_contract_voter(owner: &signer, operator: address, new_voter: address) {
        include SetStakingContractVoter;
    }

    /// Make sure staking_contract_exists first
    /// Then abort if the resource is not exist
    spec schema SetStakingContractVoter {
        use aptos_std::simple_map;
        use aptos_framework::staking_contract::{Store};

        owner: &signer;
        operator: address;
        new_voter: address;

        let owner_address = signer::address_of(owner);
        let staker = owner_address;
        let store = global<Store>(staker);
        let staking_contract_exists = exists<Store>(staker) && simple_map::spec_contains_key(store.staking_contracts, operator);
        let staker_address = owner_address;
        let staking_contract = simple_map::spec_get(store.staking_contracts, operator);
        let pool_address = staking_contract.pool_address;
        let pool_address1 = staking_contract.owner_cap.pool_address;

        aborts_if staking_contract_exists && !exists<stake::StakePool>(pool_address);
        aborts_if staking_contract_exists && !exists<stake::StakePool>(staking_contract.owner_cap.pool_address);
        
        ensures staking_contract_exists ==> global<stake::StakePool>(pool_address1).delegated_voter == new_voter;
    }

    spec set_stake_pool_voter(owner: &signer, new_voter: address) {
        include SetStakePoolVoterAbortsIf;
    }

    spec schema SetStakePoolVoterAbortsIf {
        owner: &signer;
        new_voter: address;

        let owner_address = signer::address_of(owner);
        let ownership_cap = global<stake::OwnerCapability>(owner_address);
        let pool_address = ownership_cap.pool_address;
        aborts_if stake::stake_pool_exists(owner_address) && !(exists<stake::OwnerCapability>(owner_address) && stake::stake_pool_exists(pool_address));
        ensures stake::stake_pool_exists(owner_address) ==> global<stake::StakePool>(pool_address).delegated_voter == new_voter;
    }
}
