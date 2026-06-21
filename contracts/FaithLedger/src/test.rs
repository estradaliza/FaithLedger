#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, token, Symbol};

    struct TestEnv {
        env: Env,
        contract_id: Address,
        client_contract: FaithLedgerContractClient<'static>,
        token_id: Address,
        admin: Address,
        donor: Address,
        vendor: Address,
    }

    fn setup_env() -> TestEnv {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, FaithLedgerContract);
        let client_contract = FaithLedgerContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let donor = Address::generate(&env);
        let vendor = Address::generate(&env);

        let token_id = env.register_stellar_asset_contract(Address::generate(&env));
        let token_admin = token::StellarAssetClient::new(&env, &token_id);
        
        token_admin.mint(&donor, &5000);

        TestEnv {
            env,
            contract_id,
            client_contract,
            token_id,
            admin,
            donor,
            vendor,
        }
    }

    #[test]
    fn test_1_happy_path_donation_and_distribution() {
        let t = setup_env();
        t.client_contract.initialize(&t.admin, &t.token_id);

        let fund_type = Symbol::new(&t.env, "Outreach");
        
        // Donor transfers 1000 units to Outreach bucket
        t.client_contract.donate(&t.donor, &fund_type, &1000);
        assert_eq!(t.client_contract.get_fund_balance(&fund_type), 1000);

        // Admin transparently disburses 400 to the charity vendor
        t.client_contract.distribute(&fund_type, &t.vendor, &400);
        
        let token_client = token::Client::new(&t.env, &t.token_id);
        assert_eq!(token_client.balance(&t.vendor), 400);
    }

    #[test]
    #[should_panic(expected = "Insufficient funds in the requested category allocation")]
    fn test_2_edge_case_overspending_allocation() {
        let t = setup_env();
        t.client_contract.initialize(&t.admin, &t.token_id);

        let fund_type = Symbol::new(&t.env, "Building");
        t.client_contract.donate(&t.donor, &fund_type, &500);

        // Admin attempts to spend more than what congregants allocated specifically for Building
        t.client_contract.distribute(&fund_type, &t.vendor, &600);
    }

    #[test]
    fn test_3_state_verification_of_distinct_buckets() {
        let t = setup_env();
        t.client_contract.initialize(&t.admin, &t.token_id);

        let tithe_fund = Symbol::new(&t.env, "Tithe");
        let missions_fund = Symbol::new(&t.env, "Missions");

        t.client_contract.donate(&t.donor, &tithe_fund, &800);
        t.client_contract.donate(&t.donor, &missions_fund, &200);

        // Assert state keys accurately partition separate organizational initiatives
        assert_eq!(t.client_contract.get_fund_balance(&tithe_fund), 800);
        assert_eq!(t.client_contract.get_fund_balance(&missions_fund), 200);
    }

    #[test]
    #[should_panic] // soroban require_auth triggers panic on unauthorized masquerading call
    fn test_4_unauthorized_non_admin_distribution() {
        let t = setup_env();
        t.client_contract.initialize(&t.admin, &t.token_id);

        let fund_type = Symbol::new(&t.env, "Tithe");
        t.client_contract.donate(&t.donor, &fund_type, &1000);

        // Donor tries to bypass admin approvals to spend church money directly
        t.env.as_contract(&t.contract_id, || {
            let contract_instance = FaithLedgerContract;
            contract_instance.distribute(t.env.clone(), fund_type, t.vendor, 500);
        });
    }

    #[test]
    #[should_panic(expected = "Donation amount must be positive")]
    fn test_5_edge_case_negative_donation_injection() {
        let t = setup_env();
        t.client_contract.initialize(&t.admin, &t.token_id);
        
        let fund_type = Symbol::new(&t.env, "Tithe");
        t.client_contract.donate(&t.donor, &fund_type, &-50);
    }
}