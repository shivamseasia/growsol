use crate::constants::*;
use crate::events::*;
use crate::state::presale_state::PresaleState;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_lang::solana_program::sysvar::rent::Rent;

pub fn initialize(
    ctx: Context<Initialize>,
    usd_per_sol: u64,
    presale_start_ts: i64,
    presale_end_ts: i64,
) -> Result<()> {
    let state = &mut ctx.accounts.presale_state;

    // store bumps for later verification
    state.bump = ctx.bumps.presale_state;
    state.mint_bump = ctx.bumps.mint_auth;
    state.treasury_bump = ctx.bumps.treasury;

    // basic metadata
    state.owner = ctx.accounts.owner.key();
    state.usd_per_sol = usd_per_sol;
    state.presale_start = presale_start_ts;
    state.presale_end = presale_end_ts;
    state.paused = false;
    state.current_stage = 1;

    // stage prices (in cents)
    state.stage_1_price = 1;
    state.stage_2_price = 2;
    state.stage_3_price = 3;
    state.stage_4_price = 4;
    state.stage_5_price = 5;

    // stage caps (raw units = tokens * 10^decimals)
    state.stage_1_cap = (STAGE_1_TOKENS.checked_mul(TOKEN_BASE).unwrap()) as u64;
    state.stage_2_cap = (STAGE_2_TOKENS.checked_mul(TOKEN_BASE).unwrap()) as u64;
    state.stage_3_cap = (STAGE_3_TOKENS.checked_mul(TOKEN_BASE).unwrap()) as u64;
    state.stage_4_cap = (STAGE_4_TOKENS.checked_mul(TOKEN_BASE).unwrap()) as u64;
    state.stage_5_cap = (STAGE_5_TOKENS.checked_mul(TOKEN_BASE).unwrap()) as u64;

    state.stage_1_sold = 0;
    state.stage_2_sold = 0;
    state.stage_3_sold = 0;
    state.stage_4_sold = 0;
    state.stage_5_sold = 0;

    state.total_allocated = 0;

    msg!(
        "Initialized presale_state {} and presale_token_account {}",
        state.key(),
        ctx.accounts.presale_token_account.key()
    );

    emit!(Initialized {
        owner: state.owner,
        start_ts: state.presale_start,
        end_ts: state.presale_end,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(usd_per_sol: u64, presale_start_ts: i64, presale_end_ts: i64)]
pub struct Initialize<'info> {
    /// Owner / initializer
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Presale state PDA
    #[account(
        init,
        payer = owner,
        seeds = [PRESALE_STATE_SEED],
        bump,
        space = 8 + PRESALE_SIZE
    )]
    pub presale_state: Account<'info, PresaleState>,

    /// Token mint (authority set to mint_auth PDA)
    #[account(
        init,
        payer = owner,
        mint::decimals = 9,
        mint::authority = mint_auth,
    )]
    pub mint: Account<'info, Mint>,

    /// Mint authority PDA (will be used as mint authority via seeds)
    #[account(
        init,
        seeds = [MINT_SEED],
        bump,
        payer = owner,
        space = 8, // no data, just store account
    )]
    pub mint_auth: UncheckedAccount<'info>,

    /// Treasury PDA (holds SOL)
    #[account(
        init,
        seeds = [TREASURY_SEED],
        bump,
        payer = owner,
        space = 8,
    )]
    pub treasury: UncheckedAccount<'info>,

    /// ATA owned by presale_state to hold tokens for distribution
    #[account(
        init,
        payer = owner,
        associated_token::mint = mint,
        associated_token::authority = presale_state,
    )]
    pub presale_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
