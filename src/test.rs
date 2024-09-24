#[cfg(test)]
mod tests {
    use cosmwasm_std::{ Addr, BlockInfo, Coin, Uint128, Uint256 };
    use cw_multi_test::Executor;

    use crate::integration_tests::tests::{
        expect_error,
        proper_instantiate,
        ADMIN,
        INJEX_TOKEN,
        USDT,
        USER,
    };
    use crate::msg::{ ExecuteMsg, QueryMsg, UserVesting, UserVestingInfo };
    use crate::state::{ Config, PERCENTS };

    #[test]
    fn proper_initialization() {
        let (app, contract) = proper_instantiate(true);

        let total_claimed_msg = QueryMsg::GetTotalClaimed {};
        let total_vested_msg = QueryMsg::GetTotalVested {};
        let config_msg = QueryMsg::GetConfig {};

        let total_claimed: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &total_claimed_msg)
            .unwrap();

        let total_vested: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &total_vested_msg)
            .unwrap();

        let config = app.wrap().query_wasm_smart(contract.addr(), &config_msg).unwrap();

        assert_eq!(Uint256::zero(), total_claimed);
        assert_eq!(Uint256::zero(), total_vested);
        assert_eq!(
            Config {
                instant_claim_percents: Uint256::from_u128(1500_u128), // 15%
                lock_minutes: Uint256::from_u128(5_u128),
                lock_periods: Uint256::from_u128(5_u128),
                injex_token: INJEX_TOKEN.to_string(),
            },
            config
        );
    }

    #[test]
    fn vest_not_admin() {
        let (mut app, contract) = proper_instantiate(true);

        let msg = ExecuteMsg::VestTokens { users: vec![] };

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &[]);

        assert!(res.is_err());

        let error_message = format!("Only admin");
        expect_error(res, error_message);
    }

    #[test]
    fn vest_no_funds() {
        let (mut app, contract) = proper_instantiate(true);

        let msg = ExecuteMsg::VestTokens { users: vec![] };

        let res = app.execute_contract(Addr::unchecked(ADMIN), contract.addr(), &msg, &[]);

        assert!(res.is_err());

        let error_message = format!("Invalid funds were provided");
        expect_error(res, error_message);
    }

    #[test]
    fn vest_no_users() {
        let (mut app, contract) = proper_instantiate(true);

        let msg = ExecuteMsg::VestTokens {
            users: vec![],
        };
        {
        }

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: USDT.to_string(),
                amount: Uint128::new(1_000_000),
            }]
        );

        assert!(res.is_err());

        let error_message = format!("No users");
        expect_error(res, error_message);
    }

    #[test]
    fn vest_invalid_token() {
        let (mut app, contract) = proper_instantiate(true);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::zero(),
                user: USER.to_string(),
            }],
        };
        {
        }

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: USDT.to_string(),
                amount: Uint128::new(1_000_000),
            }]
        );

        assert!(res.is_err());

        let error_message = format!("Invalid coin passed in funds");
        expect_error(res, error_message);
    }

    #[test]
    fn vest_zero_amount() {
        let (mut app, contract) = proper_instantiate(true);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::zero(),
                user: USER.to_string(),
            }],
        };
        {
        }

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount: Uint128::new(1_000_000),
            }]
        );

        assert!(res.is_err());

        let error_message = format!("Invalid user INJX amount");
        expect_error(res, error_message);
    }

    #[test]
    fn vest_invalid_funds_amount() {
        let (mut app, contract) = proper_instantiate(true);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_u128(1001_u128),
                user: USER.to_string(),
            }],
        };
        {
        }

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount: Uint128::new(1000),
            }]
        );

        assert!(res.is_err());

        let error_message = format!("Invalid funds were provided");
        expect_error(res, error_message);
    }

    #[test]
    fn vest() {
        let (mut app, contract) = proper_instantiate(true);

        let amount = Uint128::new(1_000_000);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_uint128(amount),
                user: USER.to_string(),
            }],
        };
        let balance = app.wrap().query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount,
            }]
        );

        assert!(res.is_ok());

        let balance_after = app
            .wrap()
            .query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount - amount)
        );

        let vesting_info_msg = QueryMsg::GetVestedAmount { addr: USER.to_string() };
        let claimable_amount_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let vesting_info: UserVestingInfo = app
            .wrap()
            .query_wasm_smart(contract.addr(), &vesting_info_msg)
            .unwrap();

        let claimable_amount: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &claimable_amount_msg)
            .unwrap();

        assert_eq!(
            vesting_info.next_claim,
            Uint256::from_u128(app.block_info().time.plus_minutes(5).seconds().into())
        );

        let instant_amount =
            (Uint256::from_uint128(amount) * Uint256::from_u128(1500_u128)) / PERCENTS;

        assert_eq!(claimable_amount, instant_amount);

        assert!(
            Uint256::from_uint128(amount) - vesting_info.full_amount < Uint256::from_u128(5_u128)
        );
    }

    #[test]
    fn claim_instant() {
        let (mut app, contract) = proper_instantiate(true);

        let amount = Uint128::new(1_000_000);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_uint128(amount),
                user: USER.to_string(),
            }],
        };
        let balance = app.wrap().query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount,
            }]
        );

        assert!(res.is_ok());

        let balance_after = app
            .wrap()
            .query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount - amount)
        );

        let vesting_info_msg = QueryMsg::GetVestedAmount { addr: USER.to_string() };
        let claimable_amount_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let vesting_info: UserVestingInfo = app
            .wrap()
            .query_wasm_smart(contract.addr(), &vesting_info_msg)
            .unwrap();

        let claimable_amount: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &claimable_amount_msg)
            .unwrap();

        assert_eq!(
            vesting_info.next_claim,
            Uint256::from_u128(app.block_info().time.plus_minutes(5).seconds().into())
        );

        let instant_amount =
            (Uint256::from_uint128(amount) * Uint256::from_u128(1500_u128)) / PERCENTS;

        assert_eq!(claimable_amount, instant_amount);

        assert!(
            Uint256::from_uint128(amount) - vesting_info.full_amount < Uint256::from_u128(5_u128)
        );

        let msg = ExecuteMsg::Claim {};

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + instant_amount
        );
    }

    #[test]
    fn claim_zero_reward() {
        let (mut app, contract) = proper_instantiate(true);

        let amount = Uint128::new(1_000_000);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_uint128(amount),
                user: USER.to_string(),
            }],
        };
        let balance = app.wrap().query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount,
            }]
        );

        assert!(res.is_ok());

        let balance_after = app
            .wrap()
            .query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount - amount)
        );

        let vesting_info_msg = QueryMsg::GetVestedAmount { addr: USER.to_string() };
        let claimable_amount_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let vesting_info: UserVestingInfo = app
            .wrap()
            .query_wasm_smart(contract.addr(), &vesting_info_msg)
            .unwrap();

        let claimable_amount: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &claimable_amount_msg)
            .unwrap();

        assert_eq!(
            vesting_info.next_claim,
            Uint256::from_u128(app.block_info().time.plus_minutes(5).seconds().into())
        );

        let instant_amount =
            (Uint256::from_uint128(amount) * Uint256::from_u128(1500_u128)) / PERCENTS;

        assert_eq!(claimable_amount, instant_amount);

        assert!(
            Uint256::from_uint128(amount) - vesting_info.full_amount < Uint256::from_u128(5_u128)
        );

        let msg = ExecuteMsg::Claim {};

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + instant_amount
        );

        let instant_msg = QueryMsg::GetInstantClaim { addr: USER.to_string() };

        let instant: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &instant_msg).unwrap();

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert_eq!(instant, Uint256::zero());
        assert_eq!(reward, Uint256::zero());

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        assert!(res.is_err());

        let error_message = format!("Cannot claim");
        expect_error(res, error_message);
    }

    #[test]
    fn claim_after_one_period() {
        let (mut app, contract) = proper_instantiate(true);

        let amount = Uint128::new(1_000_000);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_uint128(amount),
                user: USER.to_string(),
            }],
        };
        let balance = app.wrap().query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount,
            }]
        );

        assert!(res.is_ok());

        let balance_after = app
            .wrap()
            .query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount - amount)
        );

        let vesting_info_msg = QueryMsg::GetVestedAmount { addr: USER.to_string() };
        let claimable_amount_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let vesting_info: UserVestingInfo = app
            .wrap()
            .query_wasm_smart(contract.addr(), &vesting_info_msg)
            .unwrap();

        let claimable_amount: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &claimable_amount_msg)
            .unwrap();

        assert_eq!(
            vesting_info.next_claim,
            Uint256::from_u128(app.block_info().time.plus_minutes(5).seconds().into())
        );

        let instant_amount =
            (Uint256::from_uint128(amount) * Uint256::from_u128(1500_u128)) / PERCENTS;

        assert_eq!(claimable_amount, instant_amount);

        assert!(
            Uint256::from_uint128(amount) - vesting_info.full_amount < Uint256::from_u128(5_u128)
        );

        let msg = ExecuteMsg::Claim {};

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + instant_amount
        );

        let instant_msg = QueryMsg::GetInstantClaim { addr: USER.to_string() };

        let instant: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &instant_msg).unwrap();

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert_eq!(instant, Uint256::zero());
        assert_eq!(reward, Uint256::zero());

        let block_info = app.block_info();

        app.set_block(BlockInfo {
            chain_id: block_info.chain_id,
            height: block_info.height,
            time: block_info.time.plus_minutes(5),
        });

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert_eq!(
            reward,
            (Uint256::from_uint128(amount) - instant_amount) / Uint256::from_u128(5_u128)
        );

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + reward
        );
    }

    #[test]
    fn claim_after_two_periods() {
        let (mut app, contract) = proper_instantiate(true);

        let amount = Uint128::new(1_000_000);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_uint128(amount),
                user: USER.to_string(),
            }],
        };
        let balance = app.wrap().query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount,
            }]
        );

        assert!(res.is_ok());

        let balance_after = app
            .wrap()
            .query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount - amount)
        );

        let vesting_info_msg = QueryMsg::GetVestedAmount { addr: USER.to_string() };
        let claimable_amount_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let vesting_info: UserVestingInfo = app
            .wrap()
            .query_wasm_smart(contract.addr(), &vesting_info_msg)
            .unwrap();

        let claimable_amount: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &claimable_amount_msg)
            .unwrap();

        assert_eq!(
            vesting_info.next_claim,
            Uint256::from_u128(app.block_info().time.plus_minutes(5).seconds().into())
        );

        let instant_amount =
            (Uint256::from_uint128(amount) * Uint256::from_u128(1500_u128)) / PERCENTS;

        assert_eq!(claimable_amount, instant_amount);

        assert!(
            Uint256::from_uint128(amount) - vesting_info.full_amount < Uint256::from_u128(5_u128)
        );

        let msg = ExecuteMsg::Claim {};

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + instant_amount
        );

        let instant_msg = QueryMsg::GetInstantClaim { addr: USER.to_string() };

        let instant: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &instant_msg).unwrap();

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert_eq!(instant, Uint256::zero());
        assert_eq!(reward, Uint256::zero());

        let block_info = app.block_info();

        app.set_block(BlockInfo {
            chain_id: block_info.chain_id,
            height: block_info.height,
            time: block_info.time.plus_minutes(10),
        });

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert!(
            ((Uint256::from_uint128(amount) - instant_amount) * Uint256::from_u128(2)) /
                Uint256::from_u128(5_u128) -
                reward <= Uint256::from_u128(1)
        );

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + reward
        );
    }

    #[test]
    fn claim_full() {
        let (mut app, contract) = proper_instantiate(true);

        let amount = Uint128::new(1_000_000);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_uint128(amount),
                user: USER.to_string(),
            }],
        };
        let balance = app.wrap().query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount,
            }]
        );

        assert!(res.is_ok());

        let balance_after = app
            .wrap()
            .query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount - amount)
        );

        let vesting_info_msg = QueryMsg::GetVestedAmount { addr: USER.to_string() };
        let claimable_amount_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let vesting_info: UserVestingInfo = app
            .wrap()
            .query_wasm_smart(contract.addr(), &vesting_info_msg)
            .unwrap();

        let claimable_amount: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &claimable_amount_msg)
            .unwrap();

        assert_eq!(
            vesting_info.next_claim,
            Uint256::from_u128(app.block_info().time.plus_minutes(5).seconds().into())
        );

        let instant_amount =
            (Uint256::from_uint128(amount) * Uint256::from_u128(1500_u128)) / PERCENTS;

        assert_eq!(claimable_amount, instant_amount);

        assert!(
            Uint256::from_uint128(amount) - vesting_info.full_amount < Uint256::from_u128(5_u128)
        );

        let msg = ExecuteMsg::Claim {};

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + instant_amount
        );

        let instant_msg = QueryMsg::GetInstantClaim { addr: USER.to_string() };

        let instant: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &instant_msg).unwrap();

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert_eq!(instant, Uint256::zero());
        assert_eq!(reward, Uint256::zero());

        let block_info = app.block_info();

        app.set_block(BlockInfo {
            chain_id: block_info.chain_id,
            height: block_info.height,
            time: block_info.time.plus_minutes(25),
        });

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert!(Uint256::from_uint128(amount) - instant_amount - reward <= Uint256::from_u128(5));

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + reward
        );
    }

    #[test]
    fn claim_x2_full() {
        let (mut app, contract) = proper_instantiate(true);

        let amount = Uint128::new(1_000_000);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_uint128(amount),
                user: USER.to_string(),
            }],
        };
        let balance = app.wrap().query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount,
            }]
        );

        assert!(res.is_ok());

        let balance_after = app
            .wrap()
            .query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount - amount)
        );

        let vesting_info_msg = QueryMsg::GetVestedAmount { addr: USER.to_string() };
        let claimable_amount_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let vesting_info: UserVestingInfo = app
            .wrap()
            .query_wasm_smart(contract.addr(), &vesting_info_msg)
            .unwrap();

        let claimable_amount: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &claimable_amount_msg)
            .unwrap();

        assert_eq!(
            vesting_info.next_claim,
            Uint256::from_u128(app.block_info().time.plus_minutes(5).seconds().into())
        );

        let instant_amount =
            (Uint256::from_uint128(amount) * Uint256::from_u128(1500_u128)) / PERCENTS;

        assert_eq!(claimable_amount, instant_amount);

        assert!(
            Uint256::from_uint128(amount) - vesting_info.full_amount < Uint256::from_u128(5_u128)
        );

        let msg = ExecuteMsg::Claim {};

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + instant_amount
        );

        let instant_msg = QueryMsg::GetInstantClaim { addr: USER.to_string() };

        let instant: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &instant_msg).unwrap();

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert_eq!(instant, Uint256::zero());
        assert_eq!(reward, Uint256::zero());

        let block_info = app.block_info();

        app.set_block(BlockInfo {
            chain_id: block_info.chain_id,
            height: block_info.height,
            time: block_info.time.plus_minutes(25),
        });

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert!(Uint256::from_uint128(amount) - instant_amount - reward <= Uint256::from_u128(5));

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + reward
        );
    }

    #[test]
    fn claim_after_claim() {
        let (mut app, contract) = proper_instantiate(true);

        let amount = Uint128::new(1_000_000);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_uint128(amount),
                user: USER.to_string(),
            }],
        };
        let balance = app.wrap().query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount,
            }]
        );

        assert!(res.is_ok());

        let balance_after = app
            .wrap()
            .query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount - amount)
        );

        let vesting_info_msg = QueryMsg::GetVestedAmount { addr: USER.to_string() };
        let claimable_amount_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let vesting_info: UserVestingInfo = app
            .wrap()
            .query_wasm_smart(contract.addr(), &vesting_info_msg)
            .unwrap();

        let claimable_amount: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &claimable_amount_msg)
            .unwrap();

        assert_eq!(
            vesting_info.next_claim,
            Uint256::from_u128(app.block_info().time.plus_minutes(5).seconds().into())
        );

        let instant_amount =
            (Uint256::from_uint128(amount) * Uint256::from_u128(1500_u128)) / PERCENTS;

        assert_eq!(claimable_amount, instant_amount);

        assert!(
            Uint256::from_uint128(amount) - vesting_info.full_amount < Uint256::from_u128(5_u128)
        );

        let msg = ExecuteMsg::Claim {};

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + instant_amount
        );

        let instant_msg = QueryMsg::GetInstantClaim { addr: USER.to_string() };

        let instant: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &instant_msg).unwrap();

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert_eq!(instant, Uint256::zero());
        assert_eq!(reward, Uint256::zero());

        let block_info = app.block_info();

        app.set_block(BlockInfo {
            chain_id: block_info.chain_id,
            height: block_info.height,
            time: block_info.time.plus_minutes(10),
        });

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert!(
            ((Uint256::from_uint128(amount) - instant_amount) * Uint256::from_u128(2)) /
                Uint256::from_u128(5_u128) -
                reward <= Uint256::from_u128(5)
        );

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + reward
        );

        let block_info = app.block_info();

        app.set_block(BlockInfo {
            chain_id: block_info.chain_id,
            height: block_info.height,
            time: block_info.time.plus_minutes(15),
        });

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert!(
            ((Uint256::from_uint128(amount) - instant_amount) * Uint256::from_u128(3_u128)) /
                Uint256::from_u128(5_u128) -
                reward <= Uint256::from_u128(5)
        );

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + reward
        );
    }

    #[test]
    fn claim_after_full_claim() {
        let (mut app, contract) = proper_instantiate(true);

        let amount = Uint128::new(1_000_000);

        let msg = ExecuteMsg::VestTokens {
            users: vec![UserVesting {
                amount: Uint256::from_uint128(amount),
                user: USER.to_string(),
            }],
        };
        let balance = app.wrap().query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract.addr(),
            &msg,
            &vec![Coin {
                denom: INJEX_TOKEN.to_string(),
                amount,
            }]
        );

        assert!(res.is_ok());

        let balance_after = app
            .wrap()
            .query_balance(ADMIN.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount - amount)
        );

        let vesting_info_msg = QueryMsg::GetVestedAmount { addr: USER.to_string() };
        let claimable_amount_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let vesting_info: UserVestingInfo = app
            .wrap()
            .query_wasm_smart(contract.addr(), &vesting_info_msg)
            .unwrap();

        let claimable_amount: Uint256 = app
            .wrap()
            .query_wasm_smart(contract.addr(), &claimable_amount_msg)
            .unwrap();

        assert_eq!(
            vesting_info.next_claim,
            Uint256::from_u128(app.block_info().time.plus_minutes(5).seconds().into())
        );

        let instant_amount =
            (Uint256::from_uint128(amount) * Uint256::from_u128(1500_u128)) / PERCENTS;

        assert_eq!(claimable_amount, instant_amount);

        assert!(
            Uint256::from_uint128(amount) - vesting_info.full_amount < Uint256::from_u128(5_u128)
        );

        let msg = ExecuteMsg::Claim {};

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + instant_amount
        );

        let instant_msg = QueryMsg::GetInstantClaim { addr: USER.to_string() };

        let instant: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &instant_msg).unwrap();

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert_eq!(instant, Uint256::zero());
        assert_eq!(reward, Uint256::zero());

        let block_info = app.block_info();

        app.set_block(BlockInfo {
            chain_id: block_info.chain_id,
            height: block_info.height,
            time: block_info.time.plus_minutes(25),
        });

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert!(Uint256::from_uint128(amount) - instant_amount - reward <= Uint256::from_u128(5));

        let balance = app.wrap().query_balance(USER.to_string(), INJEX_TOKEN.to_string()).unwrap();

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        let balance_after = app
            .wrap()
            .query_balance(USER.to_string(), INJEX_TOKEN.to_string())
            .unwrap();

        assert!(res.is_ok());

        assert_eq!(
            Uint256::from_uint128(balance_after.amount),
            Uint256::from_uint128(balance.amount) + reward
        );

        let block_info = app.block_info();

        app.set_block(BlockInfo {
            chain_id: block_info.chain_id,
            height: block_info.height,
            time: block_info.time.plus_minutes(1),
        });

        let reward_msg = QueryMsg::GetClaimableAmount { addr: USER.to_string() };

        let reward: Uint256 = app.wrap().query_wasm_smart(contract.addr(), &reward_msg).unwrap();

        assert_eq!(reward, Uint256::zero());

        let res = app.execute_contract(Addr::unchecked(USER), contract.addr(), &msg, &vec![]);

        assert!(res.is_err());

        let error_message = format!("Cannot claim");
        expect_error(res, error_message);
    }
}
