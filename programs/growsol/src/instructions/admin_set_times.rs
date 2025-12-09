use anchor_lang::prelude::*;
use crate::state::presale_state::PresaleState;
use crate::events::*;
use crate::constants::*;
use crate::errors::PresaleError;

pub fn admin_set_times(ctx: Context<AdminSetTimes>, start_ts: i64, end_ts: i64) -> Result<()> {
    let state = &mut ctx.accounts.presale_state;
    require!(ctx.accounts.owner.key() == state.owner, PresaleError::Unauthorized);

    state.presale_start = start_ts;
    state.presale_end = end_ts;

    emit!(PresaleTimesUpdated { start_ts, end_ts });
    Ok(())
}

#[derive(Accounts)]
pub struct AdminSetTimes<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, seeds = [PRESALE_STATE_SEED], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,
}