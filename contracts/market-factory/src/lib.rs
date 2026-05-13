#![no_std]
 
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, Env, String, Vec, Symbol, symbol_short, log,
};
use stellarmarket_shared::{MarketStatus, SharedError};
 
// ============================================================
// STORAGE KEYS
// ============================================================
 
const KEY_MARKET_COUNT: Symbol      = symbol_short!("MKT_CNT");
const KEY_PROPOSAL_COUNT: Symbol    = symbol_short!("PROP_CNT");
const KEY_MAINTAINERS: Symbol       = symbol_short!("MNTNERS");
const KEY_THRESHOLD: Symbol         = symbol_short!("THRESHOLD");
 
// ============================================================
// DATA TYPES
// ============================================================
 
/// Metadata stored for every approved prediction market.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MarketMetadata {
    /// Unique market identifier (auto-incremented).
    pub market_id: u64,
    /// The prediction question, e.g. "Will X win the election?".
    pub question: String,
    /// Possible outcome labels, e.g. ["YES", "NO"].
    pub outcomes: Vec<String>,
    /// Unix timestamp after which trading closes and resolution begins.
    pub resolution_date: u64,
    /// Identifier of the oracle contract or provider responsible for resolution.
    pub oracle_id: Address,
    /// Current lifecycle status of the market.
    pub status: MarketStatus,
    /// Stellar address that proposed this market.
    pub proposer: Address,
    /// Ledger timestamp when the market was approved.
    pub approved_at: u64,
}
 
/// An unreviewed market proposal submitted by a user.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MarketProposal {
    /// Unique proposal identifier (auto-incremented).
    pub proposal_id: u64,
    /// The prediction question.
    pub question: String,
    /// Possible outcome labels.
    pub outcomes: Vec<String>,
    /// Requested resolution date (Unix timestamp).
    pub resolution_date: u64,
    /// Requested oracle address.
    pub oracle_id: Address,
    /// Address that submitted the proposal.
    pub proposer: Address,
    /// Ledger timestamp of submission.
    pub submitted_at: u64,
    /// Number of maintainer approvals received so far.
    pub approval_count: u32,
    /// Whether this proposal has been decided (approved or rejected).
    pub decided: bool,
}