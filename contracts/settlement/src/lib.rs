#![no_std]
 
//! # SettlementContract
//!
//! Distributes USDC winnings to holders of the winning `OutcomeToken`
//! after a market is resolved by the `OracleContract`.
//!
//! ## Payout formula
//!
//! ```
//! payout = (user_winning_tokens / total_winning_supply)
//!          * total_usdc_pool
//!          * (1 - fee_rate_bps / 10_000)
//! ```
//!
//! Where `fee_rate_bps` is in basis points (e.g. 100 = 1%).
//!
//! ## Settlement flow
//!
//! 1. Oracle finalises resolution → winning `outcome_id` known.
//! 2. Anyone calls `settle_market()` → pool locked, fee collected.
//! 3. Winners call `claim_winnings()` → tokens burned, USDC transferred.
 
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, Env, Symbol, symbol_short, log,
    token::Client as TokenClient,
};
use stellarmarket_shared::SharedError;
 
// ============================================================
// CONSTANTS
// ============================================================
 
/// Maximum fee rate in basis points (10% hard cap).
pub const MAX_FEE_RATE_BPS: u32 = 1_000;
/// Basis point denominator.
pub const BPS_DENOMINATOR: u128 = 10_000;
 
// ============================================================
// STORAGE KEYS
// ============================================================
 
const KEY_USDC_TOKEN:    Symbol = symbol_short!("USDC");
const KEY_FEE_RATE:      Symbol = symbol_short!("FEE_RATE");
const KEY_FEE_RECIPIENT: Symbol = symbol_short!("FEE_RECV");
const KEY_ORACLE:        Symbol = symbol_short!("ORACLE");
const KEY_FACTORY:       Symbol = symbol_short!("FACTORY");
 
// ============================================================
// DATA TYPES
// ============================================================
 
/// Settled state for one market.
#[contracttype]
#[derive(Clone, Debug)]
pub struct SettlementRecord {
    /// The outcome ID that won.
    pub winning_outcome_id: u32,
    /// Total USDC in the pool at settlement time.
    pub total_pool: u128,
    /// Total supply of winning outcome tokens at settlement time.
    pub total_winning_supply: u128,
    /// Protocol fee collected (in USDC).
    pub fee_collected: u128,
    /// Net pool available for winners (total_pool - fee_collected).
    pub net_pool: u128,
    /// Whether settlement has been triggered.
    pub settled: bool,
}
 
// ============================================================
// ERRORS
// ============================================================
 
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Market has not been resolved by the oracle yet.
    MarketNotResolved = 100,
    /// settle_market() has already been called for this market.
    AlreadySettled = 101,
    /// The caller holds no winning outcome tokens.
    NotAWinner = 102,
    /// The caller has already claimed their payout.
    AlreadyClaimed = 103,
    /// Settlement record not found; settle_market() must be called first.
    SettlementNotFound = 104,
    /// Fee rate exceeds the maximum allowed (1000 bps = 10%).
    FeeTooHigh = 105,
}