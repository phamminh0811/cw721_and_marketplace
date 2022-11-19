use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    #[error("NoFunds")]
    NoFunds {},

    #[error("MultipleDenoms")]
    MultipleDenoms,

    #[error("DenomNotMatch")]
    DenomNotMatch,

    #[error("PriceMustBePosiTive")]
    PriceMustBePosiTive {},

    #[error("SaleTypeMustBeFixedPrice")]
    SaleTypeMustBeFixedPrice {},

    #[error("SaleTypeMustBeAuction")]
    SaleTypeMustBeAuction {},
    
    #[error("InsufficientDeposit")]
    InsufficientDeposit {},
    
    #[error("BidExpiration")]
    BidExpiration {},

    #[error("NFTAddressNotMatch")]
    NFTAddressNotMatch {}
}
