use anchor_lang::prelude::*;

//
// Errors
//
#[error_code]
pub enum PresaleError {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Invalid oracle price")]
    InvalidOraclePrice,
    #[msg("Zero purchase amount")]
    ZeroPurchase,
    #[msg("Zero tokens allocated")]
    ZeroTokens,
    #[msg("Sale not started")]
    SaleNotStarted,
    #[msg("Sale ended")]
    SaleEnded,
    #[msg("Sale is paused")]
    SalePaused,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid stage")]
    InvalidStage,
    #[msg("Nothing to claim")]
    NothingToClaim,
    #[msg("Buyer not authorized for this allocation")]
    UnauthorizedBuyer,
    #[msg("Insufficient funds in treasury")]
    InsufficientFunds,
}

