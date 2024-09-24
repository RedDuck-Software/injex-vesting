use cosmwasm_std::Uint256;
use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{ Deserialize, Serialize };

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserVesting {
    pub amount: Uint256,
    pub user: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserVestingInfo {
    pub full_amount: Uint256,
    pub next_claim: Uint256,
    pub claimed: Uint256,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub injex_token: String,
    pub admin: String,
    pub instant_claim_percents: Uint256,
    pub lock_minutes: Uint256,
    pub lock_periods: Uint256,
}

#[cw_serde]
pub enum QueryMsg {
    GetClaimableAmount {
        addr: String,
    },
    GetInstantClaim {
        addr: String,
    },
    GetVestedAmount {
        addr: String,
    },
    GetInjxToken {},
    GetConfig {},
    GetTotalClaimed {},
    GetTotalVested {},
}

#[cw_serde]
pub enum ExecuteMsg {
    Claim {},
    VestTokens {
        users: Vec<UserVesting>,
    },
    ChangeAdmin {
        new_admin: String,
    },
    ChangeInstantClaimPercents {
        new_percents: Uint256,
    },
    ChangeLockMinutes {
        new_lock_minutes: Uint256,
    },
}
