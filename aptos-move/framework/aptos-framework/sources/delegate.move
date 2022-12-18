module aptos_framework::delegate {
    use std::bcs;
    use std::error;
    use std::signer;
    use std::vector;

    use aptos_std::math64::min;

    use aptos_framework::delegation_pool::{
    Self,
    get_stake_pool_signer,
    current_lockup_epoch,
    buy_in_active_shares,
    buy_in_inactive_shares,
    redeem_active_shares,
    redeem_inactive_shares,
    };

    use aptos_framework::account;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::coin;
    use aptos_framework::stake;
    use aptos_framework::timestamp;

    const SALT: vector<u8> = b"aptos_framework::delegate";

    const VALIDATOR_STATUS_INACTIVE: u64 = 4;

    /// Delegation pool owner capability does not exist at the provided account.
    const EOWNER_CAP_NOT_FOUND: u64 = 1;

    /// Account is already owning a delegation pool.
    const EOWNER_CAP_ALREADY_EXISTS: u64 = 2;

    const EWITHDRAW_PENDING_INACTIVE_STAKE: u64 = 3;

    /// Capability that represents ownership over not-shared operations of underlying stake pool.
    struct DelegationPoolOwnership has key, store {
        /// equal to address of the resource account owning the stake pool
        pool_address: address,
    }

    public entry fun initialize_delegation_pool(owner: &signer) {
        let owner_address = signer::address_of(owner);
        assert!(!owner_cap_exists(owner_address), error::already_exists(EOWNER_CAP_ALREADY_EXISTS));

        let seed = bcs::to_bytes(&owner_address);
        vector::append(&mut seed, SALT);
        let (stake_pool_signer, stake_pool_signer_cap) = account::create_resource_account(owner, seed);
        coin::register<AptosCoin>(&stake_pool_signer);

        // stake_pool_signer is owner account of stake pool and has `OwnerCapability`
        let pool_address = signer::address_of(&stake_pool_signer);
        stake::initialize_stake_owner(&stake_pool_signer, 0, owner_address, owner_address);

        delegation_pool::initialize(&stake_pool_signer, stake_pool_signer_cap);

        // save resource-account address (inner pool address) + outer pool ownership on `owner`
        move_to(owner, DelegationPoolOwnership { pool_address });
    }

    public fun owner_cap_exists(addr: address): bool {
        exists<DelegationPoolOwnership>(addr)
    }

    fun assert_owner_cap_exists(owner: address) {
        assert!(owner_cap_exists(owner), error::not_found(EOWNER_CAP_NOT_FOUND));
    }

    public fun get_owned_pool_address(owner: address): address acquires DelegationPoolOwnership {
        assert_owner_cap_exists(owner);
        borrow_global<DelegationPoolOwnership>(owner).pool_address
    }

    public entry fun set_operator(owner: &signer, new_operator: address) acquires DelegationPoolOwnership {
        stake::set_operator(
            &get_stake_pool_signer(get_owned_pool_address(signer::address_of(owner))),
            new_operator
        );
    }

    public entry fun set_delegated_voter(owner: &signer, new_voter: address) acquires DelegationPoolOwnership {
        stake::set_delegated_voter(
            &get_stake_pool_signer(get_owned_pool_address(signer::address_of(owner))),
            new_voter
        );
    }

    public entry fun add_stake(delegator: &signer, pool_address: address, amount: u64) {
        let stake_pool_signer = get_stake_pool_signer(pool_address);
        let delegator_address = signer::address_of(delegator);

        coin::transfer<AptosCoin>(delegator, signer::address_of(&stake_pool_signer), amount);
        stake::add_stake(&stake_pool_signer, amount);

        buy_in_active_shares(pool_address, delegator_address, amount);
    }

    public entry fun unlock(delegator: &signer, pool_address: address, amount: u64) {
        let stake_pool_signer = get_stake_pool_signer(pool_address);
        let delegator_address = signer::address_of(delegator);

        // ensure there is enough active stake on stake pool to unlock
        let (active, _, _, _) = stake::get_stake(pool_address);
        let amount = min(amount, active);

        amount = redeem_active_shares(pool_address, delegator_address, amount);
        stake::unlock(&stake_pool_signer, amount);
        buy_in_inactive_shares(pool_address, delegator_address, amount);
    }

    public entry fun reactivate_stake(delegator: &signer, pool_address: address, amount: u64) {
        let stake_pool_signer = get_stake_pool_signer(pool_address);
        let delegator_address = signer::address_of(delegator);

        let amount = redeem_inactive_shares(pool_address, delegator_address, amount, current_lockup_epoch(pool_address));
        stake::reactivate_stake(&stake_pool_signer, amount);
        buy_in_active_shares(pool_address, delegator_address, amount);
    }

    public entry fun withdraw(delegator: &signer, pool_address: address, amount: u64, lockup_epoch: u64) {
        let stake_pool_signer = get_stake_pool_signer(pool_address);
        let delegator_address = signer::address_of(delegator);

        assert!((stake::get_validator_state(pool_address) == VALIDATOR_STATUS_INACTIVE &&
                 timestamp::now_seconds() >= stake::get_lockup_secs(pool_address))
                || (lockup_epoch < current_lockup_epoch(pool_address)),
            error::invalid_argument(EWITHDRAW_PENDING_INACTIVE_STAKE));

        let amount = redeem_inactive_shares(pool_address, delegator_address, amount, lockup_epoch);
        stake::withdraw(&stake_pool_signer, amount);
        coin::transfer<AptosCoin>(&stake_pool_signer, delegator_address, amount);
    }
}
