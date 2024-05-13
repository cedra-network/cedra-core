#[test_only]
module resource_account::test_bonding_curve_launchpad {
    use aptos_std::string;
    use aptos_std::signer;
    use aptos_std::vector;
    use aptos_std::math64;
    use aptos_framework::account;
    use aptos_framework::resource_account;
    use aptos_framework::coin;
    use aptos_framework::aptos_coin::{Self, AptosCoin};
    use aptos_framework::primary_fungible_store;
    use resource_account::bonding_curve_launchpad;
    use resource_account::liquidity_pair;
    use resource_account::resource_signer_holder;
    use swap::test_helpers;


    const ELIQUIDITY_PAIR_SWAP_AMOUNTOUT_INCORRECT: u64 = 1001;
    const EUSER_APT_BALANCE_INCORRECT: u64 = 10001;
    const EINCORRECT_FROZEN_STATUS: u64 = 10002;
    const EUSER_FA_BALANCE_INCORRECT: u64 = 10003;

    //---------------------------Test Helpers---------------------------
    fun test_setup_accounts(aptos_framework: &signer, _swap_dex_signer: &signer, bcl_owner_signer: &signer, _resource_signer: &signer, bonding_curve_creator: &signer) {
        account::create_account_for_test(@0x1);
        account::create_account_for_test(@0xcafe);
        account::create_account_for_test(@0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1);
        account::create_account_for_test(@0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f);
        resource_account::create_resource_account(bcl_owner_signer, b"random4", vector::empty());
        account::create_account_for_test(@0x803);
        coin::register<AptosCoin>(bonding_curve_creator);

        let (burn_cap, mint_cap) = aptos_coin::initialize_for_test(aptos_framework);
        let bcc_coins = coin::mint(1_000_000_000_000_000, &mint_cap);
        let bcc_address = signer::address_of(bonding_curve_creator);
        coin::deposit(bcc_address, bcc_coins);

        coin::destroy_burn_cap(burn_cap);
        coin::destroy_mint_cap(mint_cap);
    }

    fun test_setup_initialize_contracts(swap_dex_signer: &signer, resource_signer: &signer) {
        test_helpers::set_up(swap_dex_signer);
        liquidity_pair::initialize_for_test(resource_signer);
        resource_signer_holder::initialize_for_test(resource_signer);
        bonding_curve_launchpad::initialize_for_test(resource_signer);
    }

    //---------------------------E2E Tests---------------------------
    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    fun test_e2e_bonding_curve_creation(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_setup_accounts(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        test_setup_initialize_contracts(swap_dex_signer, resource_signer);
        // Create FA and LiquidityPair, w.o Initial Swap.
        let user_address = signer::address_of(bonding_curve_creator);
        let starting_apt_balance = coin::balance<AptosCoin>(user_address);
        let name =  string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            0,
            name,
            symbol,
            803_000_000,
            8,
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"),
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg")
        );
        assert!(coin::balance<AptosCoin>(user_address) == starting_apt_balance, EUSER_APT_BALANCE_INCORRECT);
        assert!(bonding_curve_launchpad::get_balance(name, symbol, user_address) == 0, EUSER_FA_BALANCE_INCORRECT);
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    fun test_e2e_bonding_curve_creation_with_initial_liquidity(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_setup_accounts(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        test_setup_initialize_contracts(swap_dex_signer, resource_signer);
        // Create FA and LiquidityPair, w/ Initial Swap.
        let user_address = signer::address_of(bonding_curve_creator);
        let starting_apt_balance = coin::balance<AptosCoin>(user_address);
        let name =  string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            1_000,
            name,
            symbol,
            803_000_000,
            8,
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"),
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg")
        );
        assert!(coin::balance<AptosCoin>(user_address) == starting_apt_balance - 1000, EUSER_APT_BALANCE_INCORRECT);
        assert!(bonding_curve_launchpad::get_balance(name, symbol, user_address) == 16, EUSER_FA_BALANCE_INCORRECT);
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    fun test_e2e_bonding_curve_creation_multiple(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_setup_accounts(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        test_setup_initialize_contracts(swap_dex_signer, resource_signer);
        // Create FA and LiquidityPair, w.o Initial Swap.
        let name =  string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            0,
            name,
            symbol,
            803_000_000,
            8,
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"),
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg")
        );
        let second_fa_name =  string::utf8(b"RammyCoin");
        let second_fa_symbol = string::utf8(b"RAM");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            0,
            second_fa_name,
            second_fa_symbol,
            803_000_000,
            8,
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"),
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg")
        );
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    fun test_e2e_directional_swaps(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_bonding_curve_creation(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        let user_address = signer::address_of(bonding_curve_creator);
        let name =  string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        let starting_apt_balance = coin::balance<AptosCoin>(user_address);
        // APT -> FA
        bonding_curve_launchpad::swap_apt_to_fa(bonding_curve_creator, name, symbol, 100_000_000);
        assert!(coin::balance<AptosCoin>(user_address) == starting_apt_balance - 100_000_000, EUSER_APT_BALANCE_INCORRECT);
        assert!(bonding_curve_launchpad::get_balance(name, symbol, user_address) == 1_602_794, ELIQUIDITY_PAIR_SWAP_AMOUNTOUT_INCORRECT);
        // FA -> APT
        bonding_curve_launchpad::swap_fa_to_apt(bonding_curve_creator, name, symbol, 1_602_794);
        assert!(coin::balance<AptosCoin>(user_address) == starting_apt_balance - 26, EUSER_APT_BALANCE_INCORRECT); // u256/u64 precision loss.
        assert!(bonding_curve_launchpad::get_balance(name, symbol, user_address) == 0, ELIQUIDITY_PAIR_SWAP_AMOUNTOUT_INCORRECT);
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    fun test_e2e_graduation(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_bonding_curve_creation(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        let grad_apt: u64 = 6_000 * math64::pow(10, (8 as u64));
        let name = string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        assert!(bonding_curve_launchpad::get_is_frozen(name, symbol) == true, EINCORRECT_FROZEN_STATUS);
        bonding_curve_launchpad::swap_apt_to_fa(bonding_curve_creator, name, symbol, grad_apt); // Over-threshold Swap. APT -> FA
        assert!(bonding_curve_launchpad::get_is_frozen(name, symbol) == false, EINCORRECT_FROZEN_STATUS);
    }

    fun test_e2e_swap_after_graduation(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_graduation(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        let fa_obj_metadata = bonding_curve_launchpad::get_metadata(string::utf8(b"SheepyCoin"), string::utf8(b"SHEEP"));
        primary_fungible_store::transfer(bonding_curve_creator, fa_obj_metadata, @0xcafe, 100);
    }


    // ----E2E EXPECTED FAILING-----
    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    #[expected_failure(abort_code = 10, location = bonding_curve_launchpad)]
    fun test_e2e_failing_duplicate_FA(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_bonding_curve_creation(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator); // SheepyCoin, SHEEP
        let name =  string::utf8(b"SheepyCoin");
        let symbol = string::utf8(b"SHEEP");
        bonding_curve_launchpad::create_fa_pair(
            bonding_curve_creator,
            1_000,
            name,
            symbol,
            803_000_000,
            8,
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg"),
            string::utf8(b"https://t4.ftcdn.net/jpg/03/12/95/13/360_F_312951336_8LxW7gBLHslTnpbOAwxFo5FpD2R5vGxu.jpg")
        );
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    #[expected_failure(abort_code = 102, location = liquidity_pair)]
    fun test_e2e_failing_apt_swap_after_graduation(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_graduation(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap_apt_to_fa(bonding_curve_creator, string::utf8(b"SheepyCoin"), string::utf8(b"SHEEP"), 1_000_000); // APT -> FA
    }
    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    #[expected_failure(abort_code = 102, location = liquidity_pair)]
    fun test_e2e_failing_fa_swap_after_graduation(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_graduation(aptos_framework,swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap_fa_to_apt(bonding_curve_creator, string::utf8(b"SheepyCoin"), string::utf8(b"SHEEP"), 10); // FA -> APT
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    #[expected_failure(abort_code = 11, location = bonding_curve_launchpad)]
    fun test_e2e_failing_swap_of_nonexistant_fa(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_setup_accounts(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        test_setup_initialize_contracts(swap_dex_signer, resource_signer);
        bonding_curve_launchpad::swap_apt_to_fa(bonding_curve_creator, string::utf8(b"SheepyCoin"), string::utf8(b"SHEEP"), 1_000_000);
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    #[expected_failure(abort_code = 13, location = bonding_curve_launchpad)]
    fun test_e2e_failing_transfer_of_frozen_fa(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_bonding_curve_creation_with_initial_liquidity(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        let fa_obj_metadata = bonding_curve_launchpad::get_metadata(string::utf8(b"SheepyCoin"), string::utf8(b"SHEEP"));
        primary_fungible_store::transfer(bonding_curve_creator, fa_obj_metadata, @0xcafe, 10);
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    #[expected_failure(abort_code = 110, location = bonding_curve_launchpad)]
    fun test_e2e_failing_swap_of_zero_input_apt(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_bonding_curve_creation(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap_apt_to_fa(bonding_curve_creator, string::utf8(b"SheepyCoin"), string::utf8(b"SHEEP"), 0); // APT -> FA
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    #[expected_failure(abort_code = 110, location = bonding_curve_launchpad)]
    fun test_e2e_failing_swap_of_zero_input_fa(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_bonding_curve_creation(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap_fa_to_apt(bonding_curve_creator, string::utf8(b"SheepyCoin"), string::utf8(b"SHEEP"), 0); // Swap afer graduation, guaranteed to fail. FA -> APT
    }

    #[test(aptos_framework = @0x1, swap_dex_signer = @0xcafe, bcl_owner_signer = @0x922a028b0dbd8ff206074977ae4c5f9fb003ce384242b6253c67192cd2a45ee1, resource_signer = @0x52ddc290f7be79b2583472217af88a8500bdcb16d865e9c2bf4d3c995df0825f, bonding_curve_creator = @0x803)]
    #[expected_failure(abort_code = 12, location = liquidity_pair)]
    fun test_e2e_failing_swap_of_user_without_fa(aptos_framework: &signer, swap_dex_signer: &signer, bcl_owner_signer: &signer, resource_signer: &signer, bonding_curve_creator: &signer){
        test_e2e_bonding_curve_creation(aptos_framework, swap_dex_signer, bcl_owner_signer, resource_signer, bonding_curve_creator);
        bonding_curve_launchpad::swap_fa_to_apt(bonding_curve_creator, string::utf8(b"SheepyCoin"), string::utf8(b"SHEEP"), 10000); // Swap afer graduation, guaranteed to fail. FA -> APT
    }

}
