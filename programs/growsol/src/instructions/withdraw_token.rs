use crate::constants::*;
use crate::errors::PresaleError;
use crate::events::*;
use crate::state::presale_state::PresaleState;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, TokenAccount, Token};
use anchor_spl::associated_token::AssociatedToken;

pub fn withdraw_token(ctx: Context<WithdrawToken>, amount_raw: u64) -> Result<()> {
    let state = &ctx.accounts.presale_state;

    require!(
        ctx.accounts.owner.key() == state.owner,
        PresaleError::Unauthorized
    );

    // use mint_auth PDA as authority signer to move tokens from presale_token_account
    let mint_auth_seeds = &[MINT_SEED.as_ref(), &[state.mint_bump]];
    let signer_seeds = &[&mint_auth_seeds[..]];

    let cpi_accounts = token::Transfer {
        from: ctx.accounts.presale_token_account.to_account_info(),
        to: ctx.accounts.owner_token_account.to_account_info(),
        authority: ctx.accounts.mint_auth.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    token::transfer(cpi_ctx, amount_raw)?;

    emit!(WithdrawnToken {
        owner: state.owner,
        amount_raw,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(seeds = [PRESALE_STATE_SEED], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,

    #[account(mut, seeds = [MINT_SEED], bump = presale_state.mint_bump)]
    pub mint_auth: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// owner's ATA (will be created if needed)
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub owner_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub presale_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
