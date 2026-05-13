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
    contract, contractimpl, contracttype, contracterror,
    Address, Env, String, Symbol, symbol_short, log,
};
use stellarmarket_shared::SharedError;
 
// ============================================================
// STORAGE KEYS
// ============================================================
 
const KEY_MARKET_CONTRACT:     Symbol = symbol_short!("MKT_CTR");
const KEY_SETTLEMENT_CONTRACT: Symbol = symbol_short!("SETT_CTR");
const KEY_MARKET_ID:           Symbol = symbol_short!("MKT_ID");
const KEY_OUTCOME_ID:          Symbol = symbol_short!("OUT_ID");
const KEY_OUTCOME_LABEL:       Symbol = symbol_short!("OUT_LBL");
const KEY_TOTAL_SUPPLY:        Symbol = symbol_short!("SUPPLY");
const KEY_NAME:                Symbol = symbol_short!("NAME");
const KEY_SYMBOL:              Symbol = symbol_short!("SYMBOL");
const KEY_DECIMALS:            Symbol = symbol_short!("DECIMALS");
 
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
}