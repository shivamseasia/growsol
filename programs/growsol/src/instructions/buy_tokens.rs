use crate::constants::*;
use crate::errors::PresaleError;
use crate::events::*;
use crate::state::presale_state::PresaleState;
use crate::state::user_state::UserAllocation;
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token};

pub fn buy_tokens(ctx: Context<BuyTokens>, sol_amount: u64) -> Result<()> {
    // Logic same as your original buy_tokens function
    let clock = Clock::get()?;
    let state = &mut ctx.accounts.presale_state;

    require!(!state.paused, PresaleError::SalePaused);
    require!(
        clock.unix_timestamp >= state.presale_start,
        PresaleError::SaleNotStarted
    );
    require!(
        clock.unix_timestamp <= state.presale_end,
        PresaleError::SaleEnded
    );
    require!(state.usd_per_sol > 0, PresaleError::InvalidOraclePrice);
    require!(sol_amount > 0, PresaleError::ZeroPurchase);

    // Transfer SOL from buyer to treasury PDA (buyer is signer)
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.treasury.to_account_info(),
            },
        ),
        sol_amount,
    )?;

    // Convert lamports -> USD cents
    // usd_cents = sol_amount (lamports) * usd_per_sol * 100 / 1_000_000_000
    let sol_amount_u128 = sol_amount as u128;
    let usd_per_sol_u128 = state.usd_per_sol as u128;
    let mut usd_cents = sol_amount_u128
        .checked_mul(usd_per_sol_u128)
        .ok_or(PresaleError::MathOverflow)?
        .checked_mul(100u128)
        .ok_or(PresaleError::MathOverflow)?
        .checked_div(1_000_000_000u128)
        .ok_or(PresaleError::MathOverflow)?;

    // quick helper to get price and remaining raw cap for a stage
    fn price_and_remaining(state: &PresaleState, stage: u8) -> Result<(u64, u128)> {
        match stage {
            1 => Ok((
                state.stage_1_price,
                (state.stage_1_cap - state.stage_1_sold) as u128,
            )),
            2 => Ok((
                state.stage_2_price,
                (state.stage_2_cap - state.stage_2_sold) as u128,
            )),
            3 => Ok((
                state.stage_3_price,
                (state.stage_3_cap - state.stage_3_sold) as u128,
            )),
            4 => Ok((
                state.stage_4_price,
                (state.stage_4_cap - state.stage_4_sold) as u128,
            )),
            5 => Ok((
                state.stage_5_price,
                (state.stage_5_cap - state.stage_5_sold) as u128,
            )),
            _ => Err(PresaleError::InvalidStage.into()),
        }
    }

    let mut total_allocated_raw: u128 = 0;
    let mut stage = state.current_stage;

    // mutable reference to user allocation PDA (init_if_needed ensures it exists)
    let user_alloc = &mut ctx.accounts.user_allocation;

    while usd_cents > 0 && stage <= 5 {
        let (price_cents, remaining_raw) = price_and_remaining(state, stage)?;
        if remaining_raw == 0 {
            stage = stage.checked_add(1).ok_or(PresaleError::InvalidStage)?;
            continue;
        }

        let price_cents_u128 = price_cents as u128;

        // tokens (units) buyer can afford at this stage
        let tokens_units = usd_cents.checked_div(price_cents_u128).unwrap_or(0u128);
        if tokens_units == 0 {
            break; // not enough cents to buy a whole token unit at this stage
        }

        let raw_needed = tokens_units
            .checked_mul(TOKEN_BASE)
            .ok_or(PresaleError::MathOverflow)?;

        let to_allocate_raw = if raw_needed <= remaining_raw {
            raw_needed
        } else {
            remaining_raw
        };

        let allocated_tokens_units = to_allocate_raw
            .checked_div(TOKEN_BASE)
            .ok_or(PresaleError::MathOverflow)?;

        let usd_cents_used = allocated_tokens_units
            .checked_mul(price_cents_u128)
            .ok_or(PresaleError::MathOverflow)?;

        // update stage sold counters (in raw units)
        match stage {
            1 => {
                state.stage_1_sold = state
                    .stage_1_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?
            }
            2 => {
                state.stage_2_sold = state
                    .stage_2_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?
            }
            3 => {
                state.stage_3_sold = state
                    .stage_3_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?
            }
            4 => {
                state.stage_4_sold = state
                    .stage_4_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?
            }
            5 => {
                state.stage_5_sold = state
                    .stage_5_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?
            }
            _ => return Err(PresaleError::InvalidStage.into()),
        }

        total_allocated_raw = total_allocated_raw
            .checked_add(to_allocate_raw)
            .ok_or(PresaleError::MathOverflow)?;

        // subtract used USD cents
        usd_cents = usd_cents
            .checked_sub(usd_cents_used)
            .ok_or(PresaleError::MathOverflow)?;

        // advance stage if exhausted
        let (_, rem_after) = price_and_remaining(state, stage)?;
        if rem_after == 0 {
            stage = stage.checked_add(1).ok_or(PresaleError::InvalidStage)?;
        } else {
            continue;
        }
    }

    // clamp stage to max 5
    state.current_stage = if stage > 5 { 5 } else { stage };

    require!(total_allocated_raw > 0u128, PresaleError::ZeroTokens);

    // update totals in state (convert to u64 safely)
    state.total_allocated = state
        .total_allocated
        .checked_add(
            total_allocated_raw
                .try_into()
                .map_err(|_| PresaleError::MathOverflow)?,
        )
        .ok_or(PresaleError::MathOverflow)?;

    // update user allocation PDA: set buyer & bump verification is on the account constraint
    user_alloc.buyer = ctx.accounts.buyer.key();
    user_alloc.allocated_raw = user_alloc
        .allocated_raw
        .checked_add(
            total_allocated_raw
                .try_into()
                .map_err(|_| PresaleError::MathOverflow)?,
        )
        .ok_or(PresaleError::MathOverflow)?;

    emit!(TokensAllocated {
        buyer: user_alloc.buyer,
        allocated_raw: total_allocated_raw
            .try_into()
            .map_err(|_| PresaleError::MathOverflow)?,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// presale state (mutable)
    #[account(
        mut,
        seeds = [PRESALE_STATE_SEED],
        bump = presale_state.bump
    )]
    pub presale_state: Account<'info, PresaleState>,

    /// treasury PDA (recipient of SOL)
    #[account(
        mut,
        seeds = [TREASURY_SEED],
        bump = presale_state.treasury_bump
    )]
    pub treasury: UncheckedAccount<'info>,

    /// mint authority PDA (not signer but must be the PDA)
    #[account(
        mut,
        seeds = [MINT_SEED],
        bump = presale_state.mint_bump
    )]
    pub mint_auth: UncheckedAccount<'info>,

    /// existing mint
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// per-user allocation PDA (unique per presale_state & buyer)
    /// seeds = ["user_alloc", presale_state.key(), buyer.key()]
    #[account(
        init_if_needed,
        payer = buyer,
        seeds = [USER_ALLOC_SEED, presale_state.key().as_ref(), buyer.key().as_ref()],
        bump,
        space = 8 + USER_ALLOC_SIZE
    )]
    pub user_allocation: Account<'info, UserAllocation>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
