use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::rent::Rent;

pub mod errors;
pub mod state;
pub mod constants;
pub mod events;
pub mod instructions;

use instructions::*;

declare_id!("DjWmjS3imyiNpBVzv7LFFVZWztcYjAAXpXE2RM61oAGc");

#[program]
pub mod growsol {
    use super::*;

    /// Initialize the presale (owner creates presale state, mint, PDAs and presale ATA).
    pub fn initialize(
        ctx: Context<Initialize>,
        usd_per_sol: u64,
        presale_start_ts: i64,
        presale_end_ts: i64,
    ) -> Result<()> {
        instructions::initialize(ctx, usd_per_sol, presale_start_ts, presale_end_ts)
    }

    /// Buyer sends lamports (SOL) and receives token allocation (no immediate mint).
    pub fn buy_tokens(ctx: Context<BuyTokens>, sol_amount: u64) -> Result<()> {
        instructions::buy_tokens(ctx, sol_amount)
    }

    /// Claim function — mints any unclaimed allocated tokens into the buyer's ATA.
    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        instructions::claim_tokens(ctx)
    }

    /// Owner withdraw SOL from the treasury PDA to owner's wallet via system_program::transfer with PDA signer.
    pub fn withdraw_sol(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
        instructions::withdraw_sol(ctx, amount)
    }

    /// Owner withdraw tokens from presale vault to the owner's ATA.
    pub fn withdraw_token(ctx: Context<WithdrawToken>, amount_raw: u64) -> Result<()> {
        instructions::withdraw_token(ctx, amount_raw)
    }

    /// Admin: set presale times
    pub fn admin_set_times(
        ctx: Context<AdminSetTimes>,
        start_ts: i64,
        end_ts: i64,
    ) -> Result<()> {
        instructions::admin_set_times(ctx, start_ts, end_ts)
    }

    pub fn pause_sale(ctx: Context<AdminToggleSale>) -> Result<()> {
        instructions::pause_sale::pause_sale(ctx)
    }

    pub fn resume_sale(ctx: Context<AdminToggleSale>) -> Result<()> {
        instructions::pause_sale::resume_sale(ctx)
    }
}