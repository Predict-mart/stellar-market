
Copy

#![no_std]
 
//! # OracleContract
//!
//! Aggregates outcome reports from approved oracle providers and
//! finalises market resolutions after a configurable dispute window.
//!
//! ## Resolution flow
//!
//! 1. Market closes (past `resolution_date`).
//! 2. Registered providers call `submit_report()` within a 24h window.
//! 3. Contract checks for consensus: ≥ `consensus_threshold` fraction of
//!    providers must agree on the same `outcome_id`.
//! 4. After `dispute_window_seconds` pass with no dispute, anyone may call
//!    `finalize_resolution()` to lock in the result.
//! 5. If consensus is not reached, `dispute_resolution()` escalates to
//!    the `GovernanceContract`.
 
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, Env, Vec, Map, Symbol, symbol_short, log,
};
use stellarmarket_shared::SharedError;
 
// ============================================================
// CONFIGURATION CONSTANTS
// ============================================================
 
/// Default dispute window: 48 hours in seconds.
pub const DEFAULT_DISPUTE_WINDOW_SECS: u64 = 48 * 60 * 60;
 
/// Numerator of the required consensus fraction (2 out of 3 = 66.6%).
pub const CONSENSUS_NUMERATOR: u32 = 2;
/// Denominator of the required consensus fraction.
pub const CONSENSUS_DENOMINATOR: u32 = 3;
 
// ============================================================
// STORAGE KEYS
// ============================================================
 
const KEY_GOVERNANCE:       Symbol = symbol_short!("GOV");
const KEY_PROVIDERS:        Symbol = symbol_short!("PROVDRS");
const KEY_DISPUTE_WINDOW:   Symbol = symbol_short!("DISP_WIN");
 
// ============================================================
// DATA TYPES
// ============================================================
 
/// A single resolution report submitted by one oracle provider.
#[contracttype]
#[derive(Clone, Debug)]
pub struct OracleReport {
    /// Address of the provider submitting this report.
    pub provider: Address,
    /// Market this report pertains to.
    pub market_id: u64,
    /// The outcome the provider believes won (0-indexed).
    pub outcome_id: u32,
    /// Ledger timestamp of submission.
    pub submitted_at: u64,
    /// Provider confidence score (0–100).
    pub confidence: u8,
}
 
/// Aggregated resolution state for a market.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ResolutionState {
    /// All reports submitted for this market.
    pub reports: Vec<OracleReport>,
    /// Timestamp of the first report received.
    pub first_report_at: u64,
    /// Whether the resolution has been finalised.
    pub finalized: bool,
    /// The winning outcome, set when finalized.
    pub winning_outcome: Option<u32>,
    /// Whether a dispute has been raised.
    pub disputed: bool,
}
 
// ============================================================
// ERRORS
// ============================================================
 
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Caller is not a registered oracle provider.
    NotAProvider = 100,
    /// Provider has already submitted a report for this market.
    AlreadyReported = 101,
    /// Dispute window has not yet passed; cannot finalize yet.
    DisputeWindowActive = 102,
    /// Market resolution has already been finalised.
    AlreadyFinalized = 103,
    /// Not enough providers have reported to determine consensus.
    InsufficientReports = 104,
    /// Providers have not reached consensus on an outcome.
    NoConsensus = 105,
    /// Market resolution is already disputed.
    AlreadyDisputed = 106,
    /// Confidence value out of range (must be 0–100).
    InvalidConfidence = 107,
}