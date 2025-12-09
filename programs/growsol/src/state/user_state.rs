use anchor_lang::prelude::*;

#[account]
pub struct UserAllocation {
    pub buyer: Pubkey,
    pub allocated_raw: u64,
    pub claimed_raw: u64,
    pub bump: u8,
}