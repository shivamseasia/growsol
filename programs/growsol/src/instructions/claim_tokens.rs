use crate::constants::*;
use crate::errors::PresaleError;
use crate::events::*;
use crate::state::presale_state::PresaleState;
use crate::state::user_state::UserAllocation;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount, Token, MintTo};
use anchor_spl::associated_token::AssociatedToken;

pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
    let state = &ctx.accounts.presale_state;
    let user_alloc = &mut ctx.accounts.user_allocation;

    require!(
        user_alloc.buyer == ctx.accounts.buyer.key(),
        PresaleError::Unauthorized
    );

    let allocated = user_alloc.allocated_raw as u128;
    let claimed = user_alloc.claimed_raw as u128;
    require!(allocated > claimed, PresaleError::NothingToClaim);

    let to_claim_raw_u128 = allocated
        .checked_sub(claimed)
        .ok_or(PresaleError::MathOverflow)?;
    let to_claim_raw_u64: u64 = to_claim_raw_u128
        .try_into()
        .map_err(|_| PresaleError::MathOverflow)?;

    // mint_to using mint_auth PDA as signer
    let mint_auth_seeds = &[MINT_SEED.as_ref(), &[state.mint_bump]];
    let signer_seeds = &[&mint_auth_seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.mint_auth.to_account_info(),
            },
            signer_seeds,
        ),
        to_claim_raw_u64,
    )?;

    // mark claimed
    user_alloc.claimed_raw = user_alloc
        .claimed_raw
        .checked_add(to_claim_raw_u64)
        .ok_or(PresaleError::MathOverflow)?;

    emit!(TokensClaimed {
        buyer: user_alloc.buyer,
        claimed_raw: to_claim_raw_u64,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut, seeds = [PRESALE_STATE_SEED], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,

    /// mint auth PDA (must match presale_state.mint_bump)
    #[account(mut, seeds = [MINT_SEED], bump = presale_state.mint_bump)]
    pub mint_auth: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut, seeds = [USER_ALLOC_SEED, presale_state.key().as_ref(), buyer.key().as_ref()], bump)]
    pub user_allocation: Account<'info, UserAllocation>,

    /// buyer's ATA - will be created if missing
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = buyer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}