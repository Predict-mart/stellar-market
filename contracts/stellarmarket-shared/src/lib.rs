#![no_std]

use soroban_sdk::{contracterror, contracttype};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MarketStatus {
    Pending = 0,
    Active = 1,
    Resolved = 2,
    Settled = 3,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SharedError {
    NotAuthorized = 1,
}
