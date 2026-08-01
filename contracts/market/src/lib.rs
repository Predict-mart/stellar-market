#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
    Symbol, Vec,
};
use stellarmarket_shared::{MarketStatus, Side};

// ============================================================
// STORAGE KEYS
// ============================================================

const KEY_MARKET_ID: Symbol = symbol_short!("MKT_ID");
const KEY_QUESTION: Symbol = symbol_short!("QUESTION");
const KEY_OUTCOMES: Symbol = symbol_short!("OUTCOMES");
const KEY_STATUS: Symbol = symbol_short!("STATUS");
const KEY_RES_DATE: Symbol = symbol_short!("RES_DATE");
const KEY_ORDER_SEQ: Symbol = symbol_short!("ORD_SEQ");
const KEY_FACTORY: Symbol = symbol_short!("FACTORY");

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
    /// Caller is not authorized.
    Unauthorized = 104,
    /// Order has already been fully filled or cancelled.
    OrderNotCancellable = 105,
    /// Market is not currently active for trading.
    MarketNotActive = 106,
    /// Contract is already initialized.
    AlreadyInitialized = 107,
}

// ============================================================
// HELPER FUNCTIONS
// ============================================================

const KEY_BIDS: Symbol = symbol_short!("BIDS");
const KEY_ASKS: Symbol = symbol_short!("ASKS");

fn get_bids(env: &Env, outcome_id: u32) -> Vec<Order> {
    env.storage()
        .persistent()
        .get(&(KEY_BIDS, outcome_id))
        .unwrap_or_else(|| Vec::new(env))
}

fn set_bids(env: &Env, outcome_id: u32, bids: &Vec<Order>) {
    env.storage()
        .persistent()
        .set(&(KEY_BIDS, outcome_id), bids);
}

fn get_asks(env: &Env, outcome_id: u32) -> Vec<Order> {
    env.storage()
        .persistent()
        .get(&(KEY_ASKS, outcome_id))
        .unwrap_or_else(|| Vec::new(env))
}

fn set_asks(env: &Env, outcome_id: u32, asks: &Vec<Order>) {
    env.storage()
        .persistent()
        .set(&(KEY_ASKS, outcome_id), asks);
}

fn to_order_book_side(env: &Env, orders: &Vec<Order>) -> OrderBookSide {
    let mut levels = Vec::new(env);
    let mut current_price: Option<u64> = None;
    let mut current_qty = 0;

    for i in 0..orders.len() {
        let order = orders.get(i).unwrap();
        let rem = order.remaining();
        if rem == 0 {
            continue;
        }
        match current_price {
            None => {
                current_price = Some(order.price);
                current_qty = rem;
            }
            Some(p) => {
                if order.price == p {
                    current_qty += rem;
                } else {
                    levels.push_back(PriceLevel {
                        price: p,
                        quantity: current_qty,
                    });
                    current_price = Some(order.price);
                    current_qty = rem;
                }
            }
        }
    }

    if let Some(p) = current_price {
        if current_qty > 0 {
            levels.push_back(PriceLevel {
                price: p,
                quantity: current_qty,
            });
        }
    }

    OrderBookSide { levels }
}

// ============================================================
// CONTRACT IMPLEMENTATION
// ============================================================

#[contract]
pub struct Market;

#[contractimpl]
impl Market {
    /// Initializes the Market contract.
    pub fn initialize(
        env: Env,
        factory: Address,
        market_id: u64,
        question: String,
        outcomes: Vec<String>,
        resolution_date: u64,
    ) -> Result<(), Error> {
        if env.storage().persistent().has(&KEY_MARKET_ID) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().persistent().set(&KEY_FACTORY, &factory);
        env.storage().persistent().set(&KEY_MARKET_ID, &market_id);
        env.storage().persistent().set(&KEY_QUESTION, &question);
        env.storage().persistent().set(&KEY_OUTCOMES, &outcomes);
        env.storage()
            .persistent()
            .set(&KEY_STATUS, &MarketStatus::Active);
        env.storage()
            .persistent()
            .set(&KEY_RES_DATE, &resolution_date);
        env.storage().persistent().set(&KEY_ORDER_SEQ, &0u128);

        Ok(())
    }

    /// Places a limit order on the order book.
    pub fn place_order(
        env: Env,
        trader: Address,
        outcome_id: u32,
        side: Side,
        price: u64,
        quantity: u64,
    ) -> Result<u128, Error> {
        trader.require_auth();

        let status: MarketStatus = env
            .storage()
            .persistent()
            .get(&KEY_STATUS)
            .ok_or(Error::MarketNotActive)?;
        if status != MarketStatus::Active {
            return Err(Error::MarketNotActive);
        }

        let outcomes: Vec<String> = env
            .storage()
            .persistent()
            .get(&KEY_OUTCOMES)
            .ok_or(Error::MarketNotActive)?;
        if outcome_id >= outcomes.len() {
            return Err(Error::InvalidOutcome);
        }

        if !(1..=9999).contains(&price) {
            return Err(Error::InvalidPrice);
        }
        if quantity == 0 {
            return Err(Error::InvalidQuantity);
        }

        let order_seq: u128 = env.storage().persistent().get(&KEY_ORDER_SEQ).unwrap_or(0);
        let new_order_id = order_seq + 1;
        env.storage()
            .persistent()
            .set(&KEY_ORDER_SEQ, &new_order_id);

        let mut remaining_qty = quantity;
        let mut filled_qty = 0;

        match side {
            Side::Buy => {
                let asks = get_asks(&env, outcome_id);
                let mut updated_asks = Vec::new(&env);

                for i in 0..asks.len() {
                    let mut ask = asks.get(i).unwrap();
                    if remaining_qty == 0 || ask.price > price {
                        updated_asks.push_back(ask);
                        continue;
                    }

                    let fill = core::cmp::min(remaining_qty, ask.remaining());
                    ask.filled_quantity += fill;
                    remaining_qty -= fill;
                    filled_qty += fill;

                    // Emit TradeExecuted
                    env.events().publish(
                        (Symbol::new(&env, "TradeExecuted"), outcome_id, ask.price),
                        (new_order_id, ask.order_id, fill),
                    );

                    if !ask.is_fully_filled() {
                        updated_asks.push_back(ask);
                    }
                }
                set_asks(&env, outcome_id, &updated_asks);

                if remaining_qty > 0 {
                    let order = Order {
                        order_id: new_order_id,
                        trader: trader.clone(),
                        outcome_id,
                        side: Side::Buy,
                        price,
                        quantity,
                        filled_quantity: filled_qty,
                        timestamp: env.ledger().timestamp(),
                    };

                    let mut bids = get_bids(&env, outcome_id);
                    let mut insert_idx = bids.len();
                    for i in 0..bids.len() {
                        let bid = bids.get(i).unwrap();
                        if order.price > bid.price {
                            insert_idx = i;
                            break;
                        }
                    }
                    bids.insert(insert_idx, order);
                    set_bids(&env, outcome_id, &bids);

                    // Emit OrderPlaced
                    env.events().publish(
                        (Symbol::new(&env, "OrderPlaced"), new_order_id, trader),
                        (outcome_id, Side::Buy, price, remaining_qty),
                    );
                }
            }
            Side::Sell => {
                let bids = get_bids(&env, outcome_id);
                let mut updated_bids = Vec::new(&env);

                for i in 0..bids.len() {
                    let mut bid = bids.get(i).unwrap();
                    if remaining_qty == 0 || bid.price < price {
                        updated_bids.push_back(bid);
                        continue;
                    }

                    let fill = core::cmp::min(remaining_qty, bid.remaining());
                    bid.filled_quantity += fill;
                    remaining_qty -= fill;
                    filled_qty += fill;

                    // Emit TradeExecuted
                    env.events().publish(
                        (Symbol::new(&env, "TradeExecuted"), outcome_id, bid.price),
                        (bid.order_id, new_order_id, fill),
                    );

                    if !bid.is_fully_filled() {
                        updated_bids.push_back(bid);
                    }
                }
                set_bids(&env, outcome_id, &updated_bids);

                if remaining_qty > 0 {
                    let order = Order {
                        order_id: new_order_id,
                        trader: trader.clone(),
                        outcome_id,
                        side: Side::Sell,
                        price,
                        quantity,
                        filled_quantity: filled_qty,
                        timestamp: env.ledger().timestamp(),
                    };

                    let mut asks = get_asks(&env, outcome_id);
                    let mut insert_idx = asks.len();
                    for i in 0..asks.len() {
                        let ask = asks.get(i).unwrap();
                        if order.price < ask.price {
                            insert_idx = i;
                            break;
                        }
                    }
                    asks.insert(insert_idx, order);
                    set_asks(&env, outcome_id, &asks);

                    // Emit OrderPlaced
                    env.events().publish(
                        (Symbol::new(&env, "OrderPlaced"), new_order_id, trader),
                        (outcome_id, Side::Sell, price, remaining_qty),
                    );
                }
            }
        }

        Ok(new_order_id)
    }

    /// Cancels a resting limit order.
    pub fn cancel_order(env: Env, trader: Address, order_id: u128) -> Result<(), Error> {
        trader.require_auth();

        let status: MarketStatus = env
            .storage()
            .persistent()
            .get(&KEY_STATUS)
            .ok_or(Error::MarketNotActive)?;
        if status != MarketStatus::Active {
            return Err(Error::MarketNotActive);
        }

        let outcomes: Vec<String> = env
            .storage()
            .persistent()
            .get(&KEY_OUTCOMES)
            .ok_or(Error::MarketNotActive)?;

        for outcome_id in 0..outcomes.len() {
            // Check bids
            let bids = get_bids(&env, outcome_id);
            let mut updated_bids = Vec::new(&env);
            let mut found = false;

            for i in 0..bids.len() {
                let bid = bids.get(i).unwrap();
                if bid.order_id == order_id {
                    if bid.trader != trader {
                        return Err(Error::Unauthorized);
                    }
                    if bid.is_fully_filled() {
                        return Err(Error::OrderNotCancellable);
                    }
                    found = true;
                } else {
                    updated_bids.push_back(bid);
                }
            }

            if found {
                set_bids(&env, outcome_id, &updated_bids);
                env.events()
                    .publish((Symbol::new(&env, "OrderCancelled"), order_id, trader), ());
                return Ok(());
            }

            // Check asks
            let asks = get_asks(&env, outcome_id);
            let mut updated_asks = Vec::new(&env);

            for i in 0..asks.len() {
                let ask = asks.get(i).unwrap();
                if ask.order_id == order_id {
                    if ask.trader != trader {
                        return Err(Error::Unauthorized);
                    }
                    if ask.is_fully_filled() {
                        return Err(Error::OrderNotCancellable);
                    }
                    found = true;
                } else {
                    updated_asks.push_back(ask);
                }
            }

            if found {
                set_asks(&env, outcome_id, &updated_asks);
                env.events()
                    .publish((Symbol::new(&env, "OrderCancelled"), order_id, trader), ());
                return Ok(());
            }
        }

        let order_seq: u128 = env.storage().persistent().get(&KEY_ORDER_SEQ).unwrap_or(0);
        if order_id > order_seq {
            Err(Error::OrderNotFound)
        } else {
            Err(Error::OrderNotCancellable)
        }
    }

    /// Returns the bids and asks aggregated as PriceLevels.
    pub fn get_order_book(
        env: Env,
        outcome_id: u32,
    ) -> Result<(OrderBookSide, OrderBookSide), Error> {
        let outcomes: Vec<String> = env
            .storage()
            .persistent()
            .get(&KEY_OUTCOMES)
            .ok_or(Error::MarketNotActive)?;
        if outcome_id >= outcomes.len() {
            return Err(Error::InvalidOutcome);
        }

        let bids = get_bids(&env, outcome_id);
        let asks = get_asks(&env, outcome_id);

        Ok((
            to_order_book_side(&env, &bids),
            to_order_book_side(&env, &asks),
        ))
    }

    pub fn get_best_bid(env: Env, outcome_id: u32) -> Option<u64> {
        let bids = get_bids(&env, outcome_id);
        for i in 0..bids.len() {
            let bid = bids.get(i).unwrap();
            if bid.remaining() > 0 {
                return Some(bid.price);
            }
        }
        None
    }

    pub fn get_best_ask(env: Env, outcome_id: u32) -> Option<u64> {
        let asks = get_asks(&env, outcome_id);
        for i in 0..asks.len() {
            let ask = asks.get(i).unwrap();
            if ask.remaining() > 0 {
                return Some(ask.price);
            }
        }
        None
    }

    pub fn get_market_id(env: Env) -> Result<u64, Error> {
        env.storage()
            .persistent()
            .get(&KEY_MARKET_ID)
            .ok_or(Error::MarketNotActive)
    }

    pub fn get_question(env: Env) -> Result<String, Error> {
        env.storage()
            .persistent()
            .get(&KEY_QUESTION)
            .ok_or(Error::MarketNotActive)
    }

    pub fn get_outcomes(env: Env) -> Result<Vec<String>, Error> {
        env.storage()
            .persistent()
            .get(&KEY_OUTCOMES)
            .ok_or(Error::MarketNotActive)
    }

    pub fn get_status(env: Env) -> Result<MarketStatus, Error> {
        env.storage()
            .persistent()
            .get(&KEY_STATUS)
            .ok_or(Error::MarketNotActive)
    }

    pub fn get_resolution_date(env: Env) -> Result<u64, Error> {
        env.storage()
            .persistent()
            .get(&KEY_RES_DATE)
            .ok_or(Error::MarketNotActive)
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String, Vec};

    fn setup_market(env: &Env) -> (Address, MarketClient<'static>) {
        let contract_id = env.register_contract(None, Market);
        let client = MarketClient::new(env, &contract_id);

        let factory = Address::generate(env);
        let question = String::from_str(env, "Will Stellar hit $1?");
        let outcomes = Vec::from_array(
            env,
            [String::from_str(env, "YES"), String::from_str(env, "NO")],
        );
        let resolution_date = env.ledger().timestamp() + 3600;

        client.initialize(&factory, &1, &question, &outcomes, &resolution_date);

        (contract_id, client)
    }

    #[test]
    fn test_initialize_twice_fails() {
        let env = Env::default();
        let (_contract_id, client) = setup_market(&env);

        let factory2 = Address::generate(&env);
        let question2 = String::from_str(&env, "Will Stellar hit $1?");
        let outcomes2 = Vec::from_array(
            &env,
            [String::from_str(&env, "YES"), String::from_str(&env, "NO")],
        );
        let resolution_date2 = env.ledger().timestamp() + 3600;

        let res = client.try_initialize(&factory2, &1, &question2, &outcomes2, &resolution_date2);
        assert!(res.is_err());
    }

    #[test]
    fn test_invalid_price_and_quantity() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client) = setup_market(&env);

        let trader = Address::generate(&env);

        // Price 0 is invalid
        let res = client.try_place_order(&trader, &0, &Side::Buy, &0, &100);
        assert_eq!(res.unwrap_err(), Ok(Error::InvalidPrice));

        // Price 10000 is invalid
        let res = client.try_place_order(&trader, &0, &Side::Buy, &10000, &100);
        assert_eq!(res.unwrap_err(), Ok(Error::InvalidPrice));

        // Quantity 0 is invalid
        let res = client.try_place_order(&trader, &0, &Side::Buy, &5000, &0);
        assert_eq!(res.unwrap_err(), Ok(Error::InvalidQuantity));

        // Invalid outcome_id
        let res = client.try_place_order(&trader, &2, &Side::Buy, &5000, &100);
        assert_eq!(res.unwrap_err(), Ok(Error::InvalidOutcome));
    }

    #[test]
    fn test_place_unfilled_order_no_fill() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client) = setup_market(&env);

        let trader = Address::generate(&env);

        // Place Buy order (bids side)
        let order_id = client.place_order(&trader, &0, &Side::Buy, &5000, &100);
        assert_eq!(order_id, 1);

        // Check best bid
        assert_eq!(client.get_best_bid(&0), Some(5000));
        assert_eq!(client.get_best_ask(&0), None);

        // Check book side snapshot
        let (bids_side, asks_side) = client.get_order_book(&0);
        assert_eq!(bids_side.levels.len(), 1);
        let bid_level = bids_side.levels.get(0).unwrap();
        assert_eq!(bid_level.price, 5000);
        assert_eq!(bid_level.quantity, 100);
        assert_eq!(asks_side.levels.len(), 0);
    }

    #[test]
    fn test_full_fill() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client) = setup_market(&env);

        let maker = Address::generate(&env);
        let taker = Address::generate(&env);

        // 1. Maker places a resting sell order at 5000, quantity 100
        let ask_id = client.place_order(&maker, &0, &Side::Sell, &5000, &100);
        assert_eq!(ask_id, 1);

        // 2. Taker places a buy order at 5000, quantity 100
        let buy_id = client.place_order(&taker, &0, &Side::Buy, &5000, &100);
        assert_eq!(buy_id, 2);

        // 3. Check order book is completely cleared
        let (bids_side, asks_side) = client.get_order_book(&0);
        assert_eq!(bids_side.levels.len(), 0);
        assert_eq!(asks_side.levels.len(), 0);
        assert_eq!(client.get_best_bid(&0), None);
        assert_eq!(client.get_best_ask(&0), None);
    }

    #[test]
    fn test_partial_fill() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client) = setup_market(&env);

        let maker = Address::generate(&env);
        let taker = Address::generate(&env);

        // 1. Maker places resting sell order at 5000, quantity 100
        let ask_id = client.place_order(&maker, &0, &Side::Sell, &5000, &100);
        assert_eq!(ask_id, 1);

        // 2. Taker places a buy order at 5000, quantity 40
        let buy_id = client.place_order(&taker, &0, &Side::Buy, &5000, &40);
        assert_eq!(buy_id, 2);

        // 3. Buy order should be fully filled immediately. Remaining ask should be 60.
        let (bids_side, asks_side) = client.get_order_book(&0);
        assert_eq!(bids_side.levels.len(), 0);
        assert_eq!(asks_side.levels.len(), 1);
        let level = asks_side.levels.get(0).unwrap();
        assert_eq!(level.price, 5000);
        assert_eq!(level.quantity, 60);
    }

    #[test]
    fn test_multiple_fills_from_one_order() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client) = setup_market(&env);

        let maker1 = Address::generate(&env);
        let maker2 = Address::generate(&env);
        let maker3 = Address::generate(&env);
        let taker = Address::generate(&env);

        // 1. Place resting asks:
        // Ask 1: price 5000, quantity 50
        client.place_order(&maker1, &0, &Side::Sell, &5000, &50);
        // Ask 2: price 5100, quantity 30
        client.place_order(&maker2, &0, &Side::Sell, &5100, &30);
        // Ask 3: price 5200, quantity 40
        client.place_order(&maker3, &0, &Side::Sell, &5200, &40);

        // 2. Taker places buy order at price 5150, quantity 100
        // Should match Ask 1 (50 shares at 5000) and Ask 2 (30 shares at 5100).
        // Total matched: 80. Remaining buy order quantity: 20 at price 5150.
        // Ask 3 is untouched since its price 5200 > buy price 5150.
        let buy_id = client.place_order(&taker, &0, &Side::Buy, &5150, &100);
        assert_eq!(buy_id, 4);

        let (bids_side, asks_side) = client.get_order_book(&0);

        // Check resting bid of 20 at 5150
        assert_eq!(bids_side.levels.len(), 1);
        let bid_level = bids_side.levels.get(0).unwrap();
        assert_eq!(bid_level.price, 5150);
        assert_eq!(bid_level.quantity, 20);

        // Check resting ask of 40 at 5200
        assert_eq!(asks_side.levels.len(), 1);
        let ask_level = asks_side.levels.get(0).unwrap();
        assert_eq!(ask_level.price, 5200);
        assert_eq!(ask_level.quantity, 40);
    }

    #[test]
    fn test_cancel_order_success() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client) = setup_market(&env);

        let trader = Address::generate(&env);

        let order_id = client.place_order(&trader, &0, &Side::Buy, &5000, &100);
        assert_eq!(order_id, 1);

        // Cancel order successfully as owner
        client.cancel_order(&trader, &order_id);

        // Order book should be empty now
        let (bids, asks) = client.get_order_book(&0);
        assert_eq!(bids.levels.len(), 0);
        assert_eq!(asks.levels.len(), 0);
    }

    #[test]
    fn test_cancel_order_by_non_owner_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client) = setup_market(&env);

        let trader = Address::generate(&env);
        let non_owner = Address::generate(&env);

        let order_id = client.place_order(&trader, &0, &Side::Buy, &5000, &100);
        assert_eq!(order_id, 1);

        // Cancel by non-owner should return Error::Unauthorized
        let res = client.try_cancel_order(&non_owner, &order_id);
        assert_eq!(res.unwrap_err(), Ok(Error::Unauthorized));
    }

    #[test]
    fn test_cancel_non_existent_order_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client) = setup_market(&env);

        let trader = Address::generate(&env);

        // Try to cancel an order ID that was never created
        let res = client.try_cancel_order(&trader, &999);
        assert_eq!(res.unwrap_err(), Ok(Error::OrderNotFound));
    }

    #[test]
    fn test_cancel_already_filled_order_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let (_contract_id, client) = setup_market(&env);

        let maker = Address::generate(&env);
        let taker = Address::generate(&env);

        // Place and fill order
        let ask_id = client.place_order(&maker, &0, &Side::Sell, &5000, &100);
        let _buy_id = client.place_order(&taker, &0, &Side::Buy, &5000, &100);

        // Try to cancel the now fully filled ask order
        let res = client.try_cancel_order(&maker, &ask_id);
        assert_eq!(res.unwrap_err(), Ok(Error::OrderNotCancellable));
    }
}
