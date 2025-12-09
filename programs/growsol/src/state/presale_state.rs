use anchor_lang::prelude::*;

#[account]
pub struct PresaleState {
    pub owner: Pubkey,
    pub bump: u8,
    pub mint_bump: u8,
    pub treasury_bump: u8,

    pub usd_per_sol: u64,
    pub presale_start: i64,
    pub presale_end: i64,
    pub paused: bool,
    pub current_stage: u8,

    // prices (cents)
    pub stage_1_price: u64,
    pub stage_2_price: u64,
    pub stage_3_price: u64,
    pub stage_4_price: u64,
    pub stage_5_price: u64,

    // caps (raw)
    pub stage_1_cap: u64,
    pub stage_2_cap: u64,
    pub stage_3_cap: u64,
    pub stage_4_cap: u64,
    pub stage_5_cap: u64,

    // sold counters (raw)
    pub stage_1_sold: u64,
    pub stage_2_sold: u64,
    pub stage_3_sold: u64,
    pub stage_4_sold: u64,
    pub stage_5_sold: u64,

    pub total_allocated: u64,
}