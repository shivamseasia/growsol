use anchor_lang::prelude::*;

#[event]
pub struct Initialized {
    pub owner: Pubkey,
    pub start_ts: i64,
    pub end_ts: i64,
}

#[event]
pub struct TokensAllocated {
    pub buyer: Pubkey,
    pub allocated_raw: u64,
}

#[event]
pub struct TokensClaimed {
    pub buyer: Pubkey,
    pub claimed_raw: u64,
}

#[event]
pub struct WithdrawnSol {
    pub owner: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawnToken {
    pub owner: Pubkey,
    pub amount_raw: u64,
}

#[event]
pub struct PresaleTimesUpdated {
    pub start_ts: i64,
    pub end_ts: i64,
}

#[event]
pub struct SalePaused {
    pub owner: Pubkey,
}

#[event]
pub struct SaleResumed {
    pub owner: Pubkey,
}