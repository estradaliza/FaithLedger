#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, token, Symbol};

#[contract]
pub struct FaithLedgerContract;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Token,
    FundBalance(Symbol), // Keeps track of unique allocations like Tithe, Building, Outreach
}

#[contractimpl]
impl FaithLedgerContract {
    /// Initializes the faith organization's ledger with an admin and the accepted stable token.
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
    }

    /// Allows congregants to donate funds specifically tagged for a distinct category/bucket.
    pub fn donate(env: Env, donor: Address, fund: Symbol, amount: i128) {
        donor.require_auth();
        if amount <= 0 {
            panic!("Donation amount must be positive");
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_addr);

        // Transfer funds from donor wallet to this smart financial tracking pool
        token_client.transfer(&donor, &env.current_contract_address(), &amount);

        // Update target programmatic allocation pool balance
        let key = DataKey::FundBalance(fund.clone());
        let current_balance: i128 = env.storage().instance().get(&key).unwrap_or(0);
        env.storage().instance().set(&key, &(current_balance + amount));
    }

    /// Allows church administrators to distribute funds from a specific pool to an approved vendor/cause.
    pub fn distribute(env: Env, fund: Symbol, recipient: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if amount <= 0 {
            panic!("Distribution amount must be positive");
        }

        let key = DataKey::FundBalance(fund.clone());
        let current_balance: i128 = env.storage().instance().get(&key).unwrap_or(0);
        if current_balance < amount {
            panic!("Insufficient funds in the requested category allocation");
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_addr);

        // Execute transparent payment directly out to verified cause or vendor
        token_client.transfer(&env.current_contract_address(), &recipient, &amount);

        // Deduct allocation metrics state
        env.storage().instance().set(&key, &(current_balance - amount));
    }

    /// Read-only utility function to check balance metrics per church fund bucket.
    pub fn get_fund_balance(env: Env, fund: Symbol) -> i128 {
        let key = DataKey::FundBalance(fund);
        env.storage().instance().get(&key).unwrap_or(0)
    }
}