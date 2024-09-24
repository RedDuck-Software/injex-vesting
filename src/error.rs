use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")] Std(#[from] StdError),

    #[error("Only admin")] OnlyAdmin {},

    #[error("No users")] NoUsers {},

    #[error("Invalid percents")] InvalidPercents {},

    #[error("Invalid funds were provided")] InvalidFunds {},

    #[error("Invalid coin passed in funds")] InvalidCoin {},

    #[error("Invalid lock minutes")] InvalidLockMinutes {},

    #[error("Invalid user INJX amount")] InvalidInjxAmount {},

    #[error("Insufficient contract balance")] InsufficientContractBalance {},

    #[error("Cannot claim")] CannotClaim {},
}
