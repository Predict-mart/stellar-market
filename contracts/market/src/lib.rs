#![no_std]
 
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror,
    Address, Env, String, Vec, Map, Symbol, symbol_short, log,
};
use stellarmarket_shared::{MarketStatus, Side, SharedError};
 
// ============================================================
// STORAGE KEYS
// ============================================================
 
const KEY_MARKET_ID: Symbol     = symbol_short!("MKT_ID");
const KEY_QUESTION: Symbol      = symbol_short!("QUESTION");
const KEY_OUTCOMES: Symbol      = symbol_short!("OUTCOMES");
const KEY_STATUS: Symbol        = symbol_short!("STATUS");
const KEY_RES_DATE: Symbol      = symbol_short!("RES_DATE");
const KEY_ORDER_SEQ: Symbol     = symbol_short!("ORD_SEQ");
const KEY_FACTORY: Symbol       = symbol_short!("FACTORY");
 
// ============================================================
// DATA TYPES
// ============================================================
 
/// A resting limit order in the CLOB.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Order {
    /// Globally unique order identifier.
    pub order_id: u128,
    /// Address that placed this order.
    pub trader: Address,
    /// Which outcome this order trades (index into `outcomes` vec).
    pub outcome_id: u32,
    /// Buy or Sell.
    pub side: Side,
    /// Price in basis points (1–9999). e.g. 6500 = $0.65.
    pub price: u64,
    /// Total shares requested.
    pub quantity: u64,
    /// Shares already matched (for partial fills).
    pub filled_quantity: u64,
    /// Ledger timestamp of order placement (used for time priority).
    pub timestamp: u64,
}
 
impl Order {
    /// Remaining unfilled quantity.
    pub fn remaining(&self) -> u64 {
        self.quantity - self.filled_quantity
    }
 
    /// True if the order has been completely filled.
    pub fn is_fully_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }
}
 
/// A snapshot of a single price level in the order book.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PriceLevel {
    /// Price in basis points.
    pub price: u64,
    /// Aggregate quantity available at this price.
    pub quantity: u64,
}
 
/// Aggregated view of one side of the order book for an outcome.
#[contracttype]
#[derive(Clone, Debug)]
pub struct OrderBookSide {
    pub levels: Vec<PriceLevel>,
}
 
// ============================================================
// ERRORS
// ============================================================
 
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Price must be between 1 and 9999 basis points inclusive.
    InvalidPrice = 100,
    /// Quantity must be greater than zero.
    InvalidQuantity = 101,
    /// The given outcome_id does not exist in this market.
    InvalidOutcome = 102,
    /// Order ID does not exist.
    OrderNotFound = 103,
    /// Caller is not the order's original trader.
    NotOrderOwner = 104,
    /// Order has already been fully filled or cancelled.
    OrderNotCancellable = 105,
    /// Market is not currently active for trading.
    MarketNotActive = 106,
}