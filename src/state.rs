use schemars::JsonSchema;
use serde::{ Deserialize, Serialize };

use cosmwasm_std::{ Addr, Timestamp, Uint256 };
use cw_storage_plus::{ Item, Map };

// 100%
pub const PERCENTS: Uint256 = Uint256::from_u128(10_000_u128);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserClaims {
    pub amount: Uint256,
    pub last_claimed: Timestamp,
    pub amount_claimed: Uint256,
    pub init_vesting: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub injex_token: String,
    pub instant_claim_percents: Uint256,
    pub lock_minutes: Uint256,
    pub lock_periods: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub total_claimed: Uint256,
    pub total_vested: Uint256,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const STATE: Item<State> = Item::new("state");

pub const ADMIN: Item<Addr> = Item::new("admin");

pub const CLAIMABLE_AMOUNT: Map<Addr, UserClaims> = Map::new("claimable_amount");

pub const INSTANT_CLAIMABLE_AMOUNT: Map<Addr, Uint256> = Map::new("instant_claimable_amount");
