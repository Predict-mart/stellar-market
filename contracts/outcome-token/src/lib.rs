#![no_std]

//! # OutcomeToken
//!
//! A SEP-41 compliant fungible token representing shares in a single
//! prediction market outcome.
//!
//! - One `OutcomeToken` contract is deployed per outcome per market.
//! - Minted exclusively by the authorised `MarketContract` when a buy
//!   order is matched.
//! - Burned exclusively by the authorised `SettlementContract` when a
//!   winner redeems their payout.
//! - Freely transferable between addresses at any time.
//!
//! Winning tokens redeem 1:1 against the market's USDC pool at settlement.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol,
};

// ============================================================
// STORAGE KEYS
// ============================================================

const KEY_MARKET_CONTRACT: Symbol = symbol_short!("MKT_CTR");
const KEY_SETTLEMENT_CONTRACT: Symbol = symbol_short!("SETT_CTR");
const KEY_MARKET_ID: Symbol = symbol_short!("MKT_ID");
const KEY_OUTCOME_ID: Symbol = symbol_short!("OUT_ID");
const KEY_OUTCOME_LABEL: Symbol = symbol_short!("OUT_LBL");
const KEY_TOTAL_SUPPLY: Symbol = symbol_short!("SUPPLY");
const KEY_NAME: Symbol = symbol_short!("NAME");
const KEY_SYMBOL: Symbol = symbol_short!("SYMBOL");
const KEY_DECIMALS: Symbol = symbol_short!("DECIMALS");

// ============================================================
// ERRORS
// ============================================================

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Mint caller is not the authorised MarketContract.
    UnauthorizedMinter = 100,
    /// Burn caller is not the authorised SettlementContract.
    UnauthorizedBurner = 101,
    /// Insufficient balance for transfer or burn.
    InsufficientBalance = 102,
    /// Transfer amount must be greater than zero.
    InvalidAmount = 103,
    /// Contract is already initialized.
    AlreadyInitialized = 104,
}

// ============================================================
// DATA TYPES
// ============================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Balance(Address),
    Allowance(AllowanceDataKey),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

// TTL configuration constants
const DAY_IN_LEDGERS: u32 = 17280;
const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

// ============================================================
// HELPERS
// ============================================================

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn get_market_contract(env: &Env) -> Address {
    env.storage().instance().get(&KEY_MARKET_CONTRACT).unwrap()
}

fn get_settlement_contract(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&KEY_SETTLEMENT_CONTRACT)
        .unwrap()
}

fn get_balance(env: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    if let Some(balance) = env.storage().persistent().get::<_, i128>(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
        balance
    } else {
        0
    }
}

fn set_balance(env: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr);
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

fn get_allowance(env: &Env, from: Address, spender: Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    if let Some(allowance) = env.storage().temporary().get::<_, AllowanceValue>(&key) {
        if env.ledger().sequence() > allowance.expiration_ledger {
            AllowanceValue {
                amount: 0,
                expiration_ledger: 0,
            }
        } else {
            allowance
        }
    } else {
        AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        }
    }
}

fn set_allowance(env: &Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    let val = AllowanceValue {
        amount,
        expiration_ledger,
    };
    env.storage().temporary().set(&key, &val);
}

fn get_total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&KEY_TOTAL_SUPPLY)
        .unwrap_or(0i128)
}

fn set_total_supply(env: &Env, amount: i128) {
    env.storage().instance().set(&KEY_TOTAL_SUPPLY, &amount);
}

fn check_nonnegative_amount(env: &Env, amount: i128) {
    if amount < 0 {
        env.panic_with_error(Error::InvalidAmount);
    }
}

// ============================================================
// CONTRACT IMPLEMENTATION
// ============================================================

#[contract]
pub struct OutcomeToken;

#[contractimpl]
impl OutcomeToken {
    /// Initializes the OutcomeToken contract.
    pub fn initialize(
        env: Env,
        market_contract: Address,
        settlement_contract: Address,
        market_id: u64,
        outcome_id: u32,
        outcome_label: String,
        name: String,
        symbol: String,
        decimals: u32,
    ) -> Result<(), Error> {
        if env.storage().instance().has(&KEY_MARKET_CONTRACT) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&KEY_MARKET_CONTRACT, &market_contract);
        env.storage()
            .instance()
            .set(&KEY_SETTLEMENT_CONTRACT, &settlement_contract);
        env.storage().instance().set(&KEY_MARKET_ID, &market_id);
        env.storage().instance().set(&KEY_OUTCOME_ID, &outcome_id);
        env.storage()
            .instance()
            .set(&KEY_OUTCOME_LABEL, &outcome_label);
        env.storage().instance().set(&KEY_TOTAL_SUPPLY, &0i128);
        env.storage().instance().set(&KEY_NAME, &name);
        env.storage().instance().set(&KEY_SYMBOL, &symbol);
        env.storage().instance().set(&KEY_DECIMALS, &decimals);

        extend_instance_ttl(&env);

        Ok(())
    }

    /// Mint new tokens to an address. Only callable by the authorized MarketContract.
    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
        let market_contract = get_market_contract(&env);
        market_contract.require_auth();

        check_nonnegative_amount(&env, amount);
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }

        extend_instance_ttl(&env);

        let balance = get_balance(&env, to.clone());
        let total_supply = get_total_supply(&env);

        set_balance(&env, to.clone(), balance + amount);
        set_total_supply(&env, total_supply + amount);

        // Emit Mint event
        env.events()
            .publish((Symbol::new(&env, "mint"), market_contract, to), amount);

        Ok(())
    }

    /// Burn tokens from an address. Only callable by the authorized SettlementContract.
    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        let settlement_contract = get_settlement_contract(&env);
        settlement_contract.require_auth();

        check_nonnegative_amount(&env, amount);
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }

        extend_instance_ttl(&env);

        let balance = get_balance(&env, from.clone());
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let total_supply = get_total_supply(&env);

        set_balance(&env, from.clone(), balance - amount);
        set_total_supply(&env, total_supply - amount);

        // Emit Burn event
        env.events()
            .publish((Symbol::new(&env, "burn"), from), amount);

        Ok(())
    }

    /// Burn tokens from an address using a spender allowance.
    /// In this custom contract, only the authorized SettlementContract is allowed to burn.
    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) -> Result<(), Error> {
        let settlement_contract = get_settlement_contract(&env);
        if spender != settlement_contract {
            return Err(Error::UnauthorizedBurner);
        }
        spender.require_auth();

        check_nonnegative_amount(&env, amount);
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }

        extend_instance_ttl(&env);

        let allowance = get_allowance(&env, from.clone(), spender.clone());
        if allowance.amount < amount {
            return Err(Error::InsufficientBalance);
        }

        let balance = get_balance(&env, from.clone());
        if balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let total_supply = get_total_supply(&env);

        // Decrease allowance
        set_allowance(
            &env,
            from.clone(),
            spender.clone(),
            allowance.amount - amount,
            allowance.expiration_ledger,
        );

        set_balance(&env, from.clone(), balance - amount);
        set_total_supply(&env, total_supply - amount);

        // Emit Burn event
        env.events()
            .publish((Symbol::new(&env, "burn"), from), amount);

        Ok(())
    }

    /// Transfer tokens between addresses.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        check_nonnegative_amount(&env, amount);
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }

        extend_instance_ttl(&env);

        let from_balance = get_balance(&env, from.clone());
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let to_balance = get_balance(&env, to.clone());

        set_balance(&env, from.clone(), from_balance - amount);
        set_balance(&env, to.clone(), to_balance + amount);

        // Emit Transfer event
        env.events()
            .publish((Symbol::new(&env, "transfer"), from, to), amount);

        Ok(())
    }

    /// Transfer tokens using allowance.
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        spender.require_auth();

        check_nonnegative_amount(&env, amount);
        if amount == 0 {
            return Err(Error::InvalidAmount);
        }

        extend_instance_ttl(&env);

        let allowance = get_allowance(&env, from.clone(), spender.clone());
        if allowance.amount < amount {
            return Err(Error::InsufficientBalance);
        }

        let from_balance = get_balance(&env, from.clone());
        if from_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let to_balance = get_balance(&env, to.clone());

        // Decrease allowance
        set_allowance(
            &env,
            from.clone(),
            spender.clone(),
            allowance.amount - amount,
            allowance.expiration_ledger,
        );

        set_balance(&env, from.clone(), from_balance - amount);
        set_balance(&env, to.clone(), to_balance + amount);

        // Emit Transfer event
        env.events()
            .publish((Symbol::new(&env, "transfer"), from, to), amount);

        Ok(())
    }

    /// Returns the allowance of a spender for a holder.
    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        extend_instance_ttl(&env);
        get_allowance(&env, from, spender).amount
    }

    /// Approve a spender to transfer/burn tokens on behalf of the holder.
    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), Error> {
        from.require_auth();

        check_nonnegative_amount(&env, amount);

        extend_instance_ttl(&env);

        set_allowance(
            &env,
            from.clone(),
            spender.clone(),
            amount,
            expiration_ledger,
        );

        // Emit Approve event
        env.events()
            .publish((Symbol::new(&env, "approve"), from, spender), amount);

        Ok(())
    }

    /// Returns the token balance of an address.
    pub fn balance(env: Env, id: Address) -> i128 {
        extend_instance_ttl(&env);
        get_balance(&env, id)
    }

    /// Returns the token balance of an address (explicit acceptance criteria method).
    pub fn balance_of(env: Env, address: Address) -> i128 {
        extend_instance_ttl(&env);
        get_balance(&env, address)
    }

    /// Returns the total outstanding supply.
    pub fn total_supply(env: Env) -> i128 {
        extend_instance_ttl(&env);
        get_total_supply(&env)
    }

    /// Returns the token decimals.
    pub fn decimals(env: Env) -> u32 {
        extend_instance_ttl(&env);
        env.storage().instance().get(&KEY_DECIMALS).unwrap_or(0)
    }

    /// Returns the token name.
    pub fn name(env: Env) -> String {
        extend_instance_ttl(&env);
        env.storage().instance().get(&KEY_NAME).unwrap()
    }

    /// Returns the token symbol.
    pub fn symbol(env: Env) -> String {
        extend_instance_ttl(&env);
        env.storage().instance().get(&KEY_SYMBOL).unwrap()
    }

    // Additional contract getters for testing & introspection
    pub fn market_contract(env: Env) -> Address {
        get_market_contract(&env)
    }

    pub fn settlement_contract(env: Env) -> Address {
        get_settlement_contract(&env)
    }

    pub fn market_id(env: Env) -> u64 {
        env.storage().instance().get(&KEY_MARKET_ID).unwrap()
    }

    pub fn outcome_id(env: Env) -> u32 {
        env.storage().instance().get(&KEY_OUTCOME_ID).unwrap()
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String};

    fn setup_outcome_token(env: &Env) -> (Address, OutcomeTokenClient<'static>, Address, Address) {
        let contract_id = env.register_contract(None, OutcomeToken);
        let client = OutcomeTokenClient::new(env, &contract_id);

        let market = Address::generate(env);
        let settlement = Address::generate(env);
        let outcome_label = String::from_str(env, "YES");
        let name = String::from_str(env, "Stellar Market Outcome YES");
        let symbol = String::from_str(env, "YES");

        client.initialize(
            &market,
            &settlement,
            &1u64,
            &0u32,
            &outcome_label,
            &name,
            &symbol,
            &7u32,
        );

        (contract_id, client, market, settlement)
    }

    #[test]
    fn test_initialize_once() {
        let env = Env::default();
        let (_contract_id, client, market, settlement) = setup_outcome_token(&env);

        let outcome_label = String::from_str(&env, "YES");
        let name = String::from_str(&env, "Stellar Market Outcome YES");
        let symbol = String::from_str(&env, "YES");

        let res = client.try_initialize(
            &market,
            &settlement,
            &1u64,
            &0u32,
            &outcome_label,
            &name,
            &symbol,
            &7u32,
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_metadata() {
        let env = Env::default();
        let (_contract_id, client, market, settlement) = setup_outcome_token(&env);

        assert_eq!(
            client.name(),
            String::from_str(&env, "Stellar Market Outcome YES")
        );
        assert_eq!(client.symbol(), String::from_str(&env, "YES"));
        assert_eq!(client.decimals(), 7);
        assert_eq!(client.market_contract(), market);
        assert_eq!(client.settlement_contract(), settlement);
        assert_eq!(client.market_id(), 1);
        assert_eq!(client.outcome_id(), 0);
    }

    #[test]
    fn test_mint_success() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client, _market, _settlement) = setup_outcome_token(&env);

        let to = Address::generate(&env);
        client.mint(&to, &1000);

        assert_eq!(client.balance(&to), 1000);
        assert_eq!(client.balance_of(&to), 1000);
        assert_eq!(client.total_supply(), 1000);
    }

    #[test]
    #[should_panic]
    fn test_mint_unauthorized() {
        let env = Env::default();
        let (_contract_id, client, _market, _settlement) = setup_outcome_token(&env);
        let to = Address::generate(&env);
        // This should panic because mock_all_auths() is not called and we don't mock the market auth
        client.mint(&to, &1000);
    }

    #[test]
    fn test_burn_success() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client, _market, _settlement) = setup_outcome_token(&env);

        let to = Address::generate(&env);
        client.mint(&to, &1000);
        client.burn(&to, &400);

        assert_eq!(client.balance(&to), 600);
        assert_eq!(client.total_supply(), 600);
    }

    #[test]
    #[should_panic]
    fn test_burn_unauthorized() {
        let env = Env::default();
        let (_contract_id, client, _market, _settlement) = setup_outcome_token(&env);
        let to = Address::generate(&env);
        client.burn(&to, &400);
    }

    #[test]
    fn test_transfer_success() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client, _market, _settlement) = setup_outcome_token(&env);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        client.mint(&alice, &1000);
        client.transfer(&alice, &bob, &300);

        assert_eq!(client.balance(&alice), 700);
        assert_eq!(client.balance(&bob), 300);
        assert_eq!(client.total_supply(), 1000);
    }

    #[test]
    #[should_panic]
    fn test_transfer_insufficient_balance() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client, _market, _settlement) = setup_outcome_token(&env);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        client.mint(&alice, &100);
        client.transfer(&alice, &bob, &200);
    }

    #[test]
    fn test_allowance_and_transfer_from() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client, _market, _settlement) = setup_outcome_token(&env);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let spender = Address::generate(&env);

        client.mint(&alice, &1000);

        // Approve spender
        client.approve(&alice, &spender, &500, &100);
        assert_eq!(client.allowance(&alice, &spender), 500);

        // Transfer from
        client.transfer_from(&spender, &alice, &bob, &300);
        assert_eq!(client.balance(&alice), 700);
        assert_eq!(client.balance(&bob), 300);
        assert_eq!(client.allowance(&alice, &spender), 200);
    }

    #[test]
    #[should_panic]
    fn test_transfer_from_insufficient_allowance() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client, _market, _settlement) = setup_outcome_token(&env);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let spender = Address::generate(&env);

        client.mint(&alice, &1000);
        client.approve(&alice, &spender, &200, &100);
        client.transfer_from(&spender, &alice, &bob, &300);
    }

    #[test]
    fn test_burn_from_success() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client, _market, settlement) = setup_outcome_token(&env);

        let alice = Address::generate(&env);
        client.mint(&alice, &1000);

        // Approve settlement contract as spender
        client.approve(&alice, &settlement, &500, &100);

        // Burn from
        client.burn_from(&settlement, &alice, &300);
        assert_eq!(client.balance(&alice), 700);
        assert_eq!(client.total_supply(), 700);
        assert_eq!(client.allowance(&alice, &settlement), 200);
    }

    #[test]
    #[should_panic]
    fn test_burn_from_unauthorized_spender() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client, _market, _settlement) = setup_outcome_token(&env);

        let alice = Address::generate(&env);
        let random_spender = Address::generate(&env);
        client.mint(&alice, &1000);
        client.approve(&alice, &random_spender, &500, &100);

        // Spender is not settlement contract, so this should fail/panic
        client.burn_from(&random_spender, &alice, &300);
    }
}
