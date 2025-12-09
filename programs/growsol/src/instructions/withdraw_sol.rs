use crate::constants::*;
use crate::errors::PresaleError;
use crate::events::*;
use crate::state::presale_state::PresaleState;
use anchor_lang::prelude::*;
use anchor_lang::system_program;

pub fn withdraw_sol(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
    let state = &ctx.accounts.presale_state;

    require!(
        ctx.accounts.owner.key() == state.owner,
        PresaleError::Unauthorized
    );

    // ensure treasury has enough lamports
    let treasury_lamports = **ctx.accounts.treasury.to_account_info().lamports.borrow();
    require!(treasury_lamports >= amount, PresaleError::InsufficientFunds);

    // use PDA signer seeds for treasury PDA
    let treasury_seeds = &[TREASURY_SEED.as_ref(), &[state.treasury_bump]];
    let signer_seeds = &[&treasury_seeds[..]];

    system_program::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.treasury.to_account_info(),
                to: ctx.accounts.owner.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    emit!(WithdrawnSol {
        owner: state.owner,
        amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(seeds = [PRESALE_STATE_SEED], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,

    /// treasury PDA (signer via seeds)
    #[account(mut, seeds = [TREASURY_SEED], bump = presale_state.treasury_bump)]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
