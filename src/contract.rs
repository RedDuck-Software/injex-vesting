use std::str::FromStr;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary,
    Addr,
    BankMsg,
    Binary,
    Coin,
    CosmosMsg,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
    Timestamp,
    Uint128,
    Uint256,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg, UserVesting, UserVestingInfo };
use crate::state::{
    Config,
    State,
    UserClaims,
    ADMIN,
    CLAIMABLE_AMOUNT,
    CONFIG,
    INSTANT_CLAIMABLE_AMOUNT,
    PERCENTS,
    STATE,
};

// version info for migration info
const CONTRACT_NAME: &str = "injex-vesting";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, ContractError> {
    if msg.instant_claim_percents > PERCENTS {
        return Err(ContractError::InvalidPercents {});
    }

    if msg.lock_minutes == Uint256::zero() {
        return Err(ContractError::InvalidLockMinutes {});
    }

    let config = Config {
        injex_token: msg.injex_token,
        instant_claim_percents: msg.instant_claim_percents,
        lock_minutes: msg.lock_minutes,
        lock_periods: msg.lock_periods,
    };

    let state = State {
        total_claimed: Uint256::zero(),
        total_vested: Uint256::zero(),
    };

    let admin = msg.admin;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    ADMIN.save(deps.storage, &deps.api.addr_validate(&admin)?)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate").add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::VestTokens { users } => vest_tokens(users, deps, _env, info),
        ExecuteMsg::Claim {} => claim(deps, _env, info),
        ExecuteMsg::ChangeAdmin { new_admin } => change_admin(deps, info, new_admin),
        ExecuteMsg::ChangeLockMinutes { new_lock_minutes } =>
            change_lock_minutes(deps, info, new_lock_minutes),
        ExecuteMsg::ChangeInstantClaimPercents { new_percents } =>
            change_instant_percents(deps, info, new_percents),
    }
}

pub fn change_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String
) -> Result<Response, ContractError> {
    check_is_admin(deps.as_ref(), info.sender)?;

    ADMIN.save(deps.storage, &deps.api.addr_validate(&new_admin)?)?;

    Ok(Response::new().add_attribute("method", "change_admin"))
}

pub fn change_lock_minutes(
    deps: DepsMut,
    info: MessageInfo,
    new_lock_minutes: Uint256
) -> Result<Response, ContractError> {
    check_is_admin(deps.as_ref(), info.sender)?;

    if new_lock_minutes == Uint256::zero() {
        return Err(ContractError::InvalidLockMinutes {});
    }

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, StdError> {
            config.lock_minutes = new_lock_minutes;

            Ok(config)
        }
    )?;

    Ok(Response::new().add_attribute("method", "change_lock_minutes"))
}

pub fn change_instant_percents(
    deps: DepsMut,
    info: MessageInfo,
    new_instant_percents: Uint256
) -> Result<Response, ContractError> {
    check_is_admin(deps.as_ref(), info.sender)?;

    if new_instant_percents > PERCENTS {
        return Err(ContractError::InvalidPercents {});
    }

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, StdError> {
            config.instant_claim_percents = new_instant_percents;

            Ok(config)
        }
    )?;

    Ok(Response::new().add_attribute("method", "change_instant_percents"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetInjxToken {} => to_json_binary(&query_injx_token(deps)?),
        QueryMsg::GetClaimableAmount { addr } =>
            to_json_binary(&query_claimable_amount(deps, _env, addr)?),
        QueryMsg::GetInstantClaim { addr } => to_json_binary(&query_instant_amount(deps, addr)?),
        QueryMsg::GetTotalClaimed {} => to_json_binary(&query_total_claimed(deps)?),
        QueryMsg::GetTotalVested {} => to_json_binary(&query_total_vested(deps)?),
        QueryMsg::GetVestedAmount { addr } =>
            to_json_binary(&query_user_vesting_info(deps, _env, addr)?),
        QueryMsg::GetConfig {} => to_json_binary(&query_config(deps)?),
    }
}

pub fn vest_tokens(
    users: Vec<UserVesting>,
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {
    check_is_admin(deps.as_ref(), info.sender)?;

    let mut new_total_amount = Uint256::zero();

    if info.funds.len() != 1 {
        return Err(ContractError::InvalidFunds {});
    }

    if users.len() == 0 {
        return Err(ContractError::NoUsers {});
    }

    let config = CONFIG.load(deps.storage).unwrap();

    let coin = &info.funds[0];

    if coin.denom != config.injex_token {
        return Err(ContractError::InvalidCoin {});
    }

    for user in users {
        if user.amount == Uint256::zero() {
            return Err(ContractError::InvalidInjxAmount {});
        }

        let instant_claim_amount = (user.amount * config.instant_claim_percents) / PERCENTS;

        INSTANT_CLAIMABLE_AMOUNT.save(
            deps.storage,
            Addr::unchecked(user.user.clone()),
            &instant_claim_amount
        )?;

        let remaining_amount = user.amount - instant_claim_amount;

        let current_time = env.block.time;

        CLAIMABLE_AMOUNT.save(
            deps.storage,
            Addr::unchecked(user.user.clone()),
            &(UserClaims {
                amount: remaining_amount / config.lock_periods,
                last_claimed: current_time,
                amount_claimed: Uint256::zero(),
                init_vesting: current_time,
            })
        )?;

        new_total_amount += user.amount;
    }

    if new_total_amount != Uint256::from_uint128(info.funds[0].amount) {
        return Err(ContractError::InvalidFunds {});
    }

    STATE.update(
        deps.storage,
        |mut state| -> Result<State, StdError> {
            state.total_vested += new_total_amount;

            Ok(state)
        }
    ).unwrap();

    Ok(Response::new().add_attribute("method", "vest"))
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let sender = info.sender;
    let config = CONFIG.load(deps.storage).unwrap();

    let instant_claim = INSTANT_CLAIMABLE_AMOUNT.load(deps.storage, sender.clone()).unwrap_or(
        Uint256::zero()
    );

    let claim = CLAIMABLE_AMOUNT.load(deps.storage, sender.clone()).unwrap_or(UserClaims {
        amount: Uint256::zero(),
        last_claimed: Timestamp::from_seconds(0),
        amount_claimed: Uint256::zero(),
        init_vesting: Timestamp::from_seconds(0),
    });

    let curr_time = env.block.time;

    if claim.amount_claimed == claim.amount * config.lock_periods {
        return Err(ContractError::CannotClaim {});
    }

    let periods = u64::from_str(&(claim.amount_claimed / claim.amount).to_string()).unwrap();

    let reward = calculate_reward_amount(
        claim.amount,
        curr_time.seconds(),
        claim.init_vesting.seconds(),
        periods,
        u64::from_str(&config.lock_minutes.to_string()).unwrap(),
        claim.amount * config.lock_periods - claim.amount_claimed
    ).unwrap();

    if reward == Uint256::zero() && instant_claim == Uint256::zero() {
        return Err(ContractError::CannotClaim {});
    }

    let balance_res = deps.querier.query_balance(
        env.clone().contract.address.to_string(),
        config.injex_token.clone()
    )?;

    let balance = balance_res.amount;

    if Uint256::from_uint128(balance) < reward + instant_claim {
        return Err(ContractError::InsufficientContractBalance {});
    }

    let send_msg = BankMsg::Send {
        to_address: sender.clone().to_string(),
        amount: vec![Coin {
            amount: Uint128::from_str(&(reward + instant_claim).to_string())?,
            denom: config.injex_token.to_string(),
        }],
    };

    if instant_claim != Uint256::zero() {
        INSTANT_CLAIMABLE_AMOUNT.update(
            deps.storage,
            sender.clone(),
            |_| -> Result<Uint256, StdError> { Ok(Uint256::zero()) }
        )?;
    }

    CLAIMABLE_AMOUNT.update(
        deps.storage,
        sender.clone(),
        |claim| -> Result<UserClaims, StdError> {
            let mut claim = claim.unwrap();
            claim.last_claimed = curr_time;
            claim.amount_claimed += reward;

            Ok(claim)
        }
    )?;

    STATE.update(
        deps.storage,
        |mut state| -> Result<State, StdError> {
            state.total_claimed += reward + instant_claim;

            Ok(state)
        }
    ).unwrap();

    Ok(
        Response::new()
            .add_message(CosmosMsg::Bank(send_msg))
            .add_attribute("user", sender.clone())
            .add_attribute("amount_claimed", reward + instant_claim)
            .add_attribute("method", "execute_claim")
    )
}

pub fn query_claimable_amount(deps: Deps, env: Env, addr: String) -> StdResult<Uint256> {
    let user = Addr::unchecked(addr);
    let config = CONFIG.load(deps.storage).unwrap();

    let instant_claim = INSTANT_CLAIMABLE_AMOUNT.load(deps.storage, user.clone()).unwrap_or(
        Uint256::zero()
    );

    let claim = CLAIMABLE_AMOUNT.load(deps.storage, user.clone()).unwrap_or(UserClaims {
        amount: Uint256::zero(),
        last_claimed: Timestamp::from_seconds(0),
        amount_claimed: Uint256::zero(),
        init_vesting: Timestamp::from_seconds(0),
    });

    let curr_time = env.block.time;

    if claim.amount_claimed == claim.amount * config.lock_periods {
        return Ok(Uint256::zero());
    }

    let periods = u64::from_str(&(claim.amount_claimed / claim.amount).to_string()).unwrap();

    let reward = calculate_reward_amount(
        claim.amount,
        curr_time.seconds(),
        claim.init_vesting.seconds(),
        periods,
        u64::from_str(&config.lock_minutes.to_string()).unwrap(),
        claim.amount * config.lock_periods - claim.amount_claimed
    ).unwrap();

    Ok(reward + instant_claim)
}

pub fn query_total_claimed(deps: Deps) -> StdResult<Addr> {
    let state = STATE.load(deps.storage).unwrap();

    Ok(Addr::unchecked(state.total_claimed))
}

pub fn query_instant_amount(deps: Deps, addr: String) -> StdResult<Uint256> {
    let amount = INSTANT_CLAIMABLE_AMOUNT.load(deps.storage, Addr::unchecked(addr)).unwrap();

    Ok(amount)
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage).unwrap();

    Ok(config)
}

pub fn query_user_vesting_info(deps: Deps, env: Env, addr: String) -> StdResult<UserVestingInfo> {
    let user = Addr::unchecked(addr);
    let config = CONFIG.load(deps.storage).unwrap();
    let vesting = CLAIMABLE_AMOUNT.load(deps.storage, user.clone()).unwrap();
    let instant = INSTANT_CLAIMABLE_AMOUNT.load(deps.storage, user.clone()).unwrap();

    let lock_minutes = u64::from_str(&config.lock_minutes.to_string()).unwrap();
    let remaining_reward = vesting.amount * config.lock_periods - vesting.amount_claimed;

    let curr_time = env.block.time.seconds();
    let time_passed = (curr_time - vesting.init_vesting.seconds()) / 60;
    let periods: u64 = u64
        ::from_str(&(vesting.amount_claimed / vesting.amount).to_string())
        .unwrap();

    let next_claim = if time_passed - periods * lock_minutes >= lock_minutes {
        Uint256::zero()
    } else {
        Uint256::from_u128(
            (vesting.init_vesting.seconds() + lock_minutes * (periods + 1) * 60).into()
        )
    };

    Ok(UserVestingInfo {
        full_amount: remaining_reward + instant,
        next_claim,
        claimed: vesting.amount_claimed,
    })
}

pub fn query_total_vested(deps: Deps) -> StdResult<Addr> {
    let state = STATE.load(deps.storage).unwrap();

    Ok(Addr::unchecked(state.total_vested))
}

pub fn query_injx_token(deps: Deps) -> StdResult<Addr> {
    let config: Config = CONFIG.load(deps.storage).unwrap();

    Ok(Addr::unchecked(config.injex_token))
}

fn calculate_reward_amount(
    reward: Uint256,
    curr_time: u64,
    instant_time: u64,
    periods_claimed: u64,
    period_time: u64,
    max_reward: Uint256
) -> StdResult<Uint256> {
    let minutes_passed = (curr_time - instant_time) / 60;

    let periods_passed = minutes_passed / period_time - periods_claimed;

    let mut full_reward = reward * Uint256::from_u128(periods_passed.into());

    if full_reward > max_reward {
        full_reward = max_reward;
    }

    Ok(full_reward)
}

fn check_is_admin(deps: Deps, addr: Addr) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    let is_admin = addr == admin;

    if !is_admin {
        return Err(ContractError::OnlyAdmin {});
    }

    Ok(Response::new())
}
