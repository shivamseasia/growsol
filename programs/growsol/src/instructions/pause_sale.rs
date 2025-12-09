use crate::errors::PresaleError;
use crate::events::*;
use crate::state::presale_state::PresaleState;
use anchor_lang::prelude::*;
use crate::constants::*;

pub fn pause_sale(ctx: Context<AdminToggleSale>) -> Result<()> {
    let state = &mut ctx.accounts.presale_state;
    require!(
        ctx.accounts.owner.key() == state.owner,
        PresaleError::Unauthorized
    );
    state.paused = true;
    emit!(SalePaused { owner: state.owner });
    Ok(())
}

pub fn resume_sale(ctx: Context<AdminToggleSale>) -> Result<()> {
    let state = &mut ctx.accounts.presale_state;
    require!(
        ctx.accounts.owner.key() == state.owner,
        PresaleError::Unauthorized
    );
    state.paused = false;
    emit!(SaleResumed { owner: state.owner });
    Ok(())
}

#[derive(Accounts)]
pub struct AdminToggleSale<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, seeds = [PRESALE_STATE_SEED], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,
}
