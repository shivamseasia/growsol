use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::rent::Rent;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("6Mn8wPAxmjeJxJ3YdtMTChA2RTuQF4nFZM6EJ3H34STD");

pub const TOKEN_DECIMALS: u64 = 9;
pub const TOKEN_BASE: u128 = 1_000_000_000u128; // 10^9

#[program]
pub mod growsol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, usd_per_sol: u64) -> Result<()> {
        let state = &mut ctx.accounts.presale_state;

        // Save bumps for PDAs
        state.bump = ctx.bumps.presale_state;
        state.mint_bump = ctx.bumps.mint_auth;

        state.owner = ctx.accounts.owner.key();
        state.usd_per_sol = usd_per_sol; // interpreted as whole USD (dollars), e.g. 20 => $20
        state.current_stage = 1;

        // Stage prices in CENTS (1 = $0.01)
        state.stage_1_price = 1; // $0.01
        state.stage_2_price = 2; // $0.02
        state.stage_3_price = 3; // $0.03
        state.stage_4_price = 4; // $0.04
        state.stage_5_price = 5; // $0.05

        // Stage caps in RAW token units (token units * 10^decimals)
        state.stage_1_cap = 150_000_000u128
            .checked_mul(TOKEN_BASE)
            .ok_or(PresaleError::MathOverflow)? as u64;
        state.stage_2_cap = 200_000_000u128
            .checked_mul(TOKEN_BASE)
            .ok_or(PresaleError::MathOverflow)? as u64;
        state.stage_3_cap = 200_000_000u128
            .checked_mul(TOKEN_BASE)
            .ok_or(PresaleError::MathOverflow)? as u64;
        state.stage_4_cap = 225_000_000u128
            .checked_mul(TOKEN_BASE)
            .ok_or(PresaleError::MathOverflow)? as u64;
        state.stage_5_cap = 225_000_000u128
            .checked_mul(TOKEN_BASE)
            .ok_or(PresaleError::MathOverflow)? as u64;

        // sold counters
        state.stage_1_sold = 0;
        state.stage_2_sold = 0;
        state.stage_3_sold = 0;
        state.stage_4_sold = 0;
        state.stage_5_sold = 0;

        Ok(())
    }

    /// Flexible buy: will allocate across stages, spilling over to next if current stage cap is exceeded.
    /// `sol_amount` is in lamports (1 SOL = 1_000_000_000).
    pub fn buy_tokens(ctx: Context<BuyTokens>, sol_amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.presale_state;
        require!(state.usd_per_sol > 0, PresaleError::InvalidOraclePrice);
        require!(sol_amount > 0, PresaleError::ZeroTokens);

        // 1) Transfer SOL from buyer -> treasury (buyer signs). No PDA signer required.
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

        // 2) Convert lamports -> USD cents
        // usd_per_sol is dollars (e.g. 20), convert to cents by *100
        // usd_cents = sol_amount(lamports) * usd_per_sol * 100 / 1_000_000_000
        let sol_amount_u128 = sol_amount as u128;
        let usd_per_sol_u128 = state.usd_per_sol as u128;
        let mut usd_cents = sol_amount_u128
            .checked_mul(usd_per_sol_u128)
            .ok_or(PresaleError::MathOverflow)?
            .checked_mul(100u128)
            .ok_or(PresaleError::MathOverflow)?
            .checked_div(1_000_000_000u128)
            .ok_or(PresaleError::MathOverflow)?; // in cents

        // 3) Iterate stages, allocate tokens from usd_cents until exhausted or no stages left
        // We'll mint in RAW units (tokens * 10^decimals)
        // Helper closure to get price cents & remaining cap
        fn stage_price_and_remaining(
            state: &PresaleState,
            stage: u8,
        ) -> Result<(u64, u64)> {
            match stage {
                1 => Ok((state.stage_1_price, state.stage_1_cap - state.stage_1_sold)),
                2 => Ok((state.stage_2_price, state.stage_2_cap - state.stage_2_sold)),
                3 => Ok((state.stage_3_price, state.stage_3_cap - state.stage_3_sold)),
                4 => Ok((state.stage_4_price, state.stage_4_cap - state.stage_4_sold)),
                5 => Ok((state.stage_5_price, state.stage_5_cap - state.stage_5_sold)),
                _ => Err(PresaleError::InvalidStage.into()),
            }
        }

        // total minted raw across this buy (for bookkeeping if needed)
        let mut total_minted_raw: u128 = 0;

        // start from current stage
        let mut stage = state.current_stage;
        while usd_cents > 0 && stage <= 5 {
            let (price_cents, remaining_raw_u64) =
                stage_price_and_remaining(state, stage)?; // price in cents, remaining in raw units
            let remaining_raw: u128 = remaining_raw_u64 as u128;

            if remaining_raw == 0 {
                // advance stage
                stage = stage.checked_add(1).ok_or(PresaleError::InvalidStage)?;
                continue;
            }

            // tokens we can buy at this stage given usd_cents
            // tokens_units = usd_cents / price_cents
            // raw_needed = tokens_units * 10^decimals
            let price_cents_u128 = price_cents as u128;
            let tokens_units = usd_cents.checked_div(price_cents_u128).unwrap_or(0u128);
            if tokens_units == 0 {
                // Can't buy even one token at this stage price—stop
                break;
            }

            let raw_needed = tokens_units
                .checked_mul(TOKEN_BASE)
                .ok_or(PresaleError::MathOverflow)?;

            // allocate min(raw_needed, remaining_raw)
            let to_mint_raw = if raw_needed <= remaining_raw {
                raw_needed
            } else {
                remaining_raw
            };

            // actual tokens_units allocated at this stage
            let allocated_tokens_units = to_mint_raw
                .checked_div(TOKEN_BASE)
                .ok_or(PresaleError::MathOverflow)?;

            // compute usd_cents_used = allocated_tokens_units * price_cents
            let usd_cents_used = allocated_tokens_units
                .checked_mul(price_cents_u128)
                .ok_or(PresaleError::MathOverflow)?;

            // Mint `to_mint_raw` tokens to the buyer's ATA (raw units)
            // Use mint_auth PDA as signer
            let mint_auth_seeds = &[b"mint_auth".as_ref(), &[state.mint_bump][..]];
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
                to_mint_raw
                    .try_into()
                    .map_err(|_| PresaleError::MathOverflow)?,
            )?;

            // Update stage sold counters
            match stage {
                1 => state.stage_1_sold = state
                    .stage_1_sold
                    .checked_add(to_mint_raw.try_into().unwrap())
                    .ok_or(PresaleError::MathOverflow)?,
                2 => state.stage_2_sold = state
                    .stage_2_sold
                    .checked_add(to_mint_raw.try_into().unwrap())
                    .ok_or(PresaleError::MathOverflow)?,
                3 => state.stage_3_sold = state
                    .stage_3_sold
                    .checked_add(to_mint_raw.try_into().unwrap())
                    .ok_or(PresaleError::MathOverflow)?,
                4 => state.stage_4_sold = state
                    .stage_4_sold
                    .checked_add(to_mint_raw.try_into().unwrap())
                    .ok_or(PresaleError::MathOverflow)?,
                5 => state.stage_5_sold = state
                    .stage_5_sold
                    .checked_add(to_mint_raw.try_into().unwrap())
                    .ok_or(PresaleError::MathOverflow)?,
                _ => return Err(PresaleError::InvalidStage.into()),
            }

            total_minted_raw = total_minted_raw
                .checked_add(to_mint_raw)
                .ok_or(PresaleError::MathOverflow)?;

            // subtract used USD cents
            usd_cents = usd_cents
                .checked_sub(usd_cents_used)
                .ok_or(PresaleError::MathOverflow)?;

            // Advance stage if this stage is fully sold
            let remaining_after = {
                let (_, rem_after) = stage_price_and_remaining(state, stage)?;
                rem_after as u128
            };
            if remaining_after == 0 {
                stage = stage.checked_add(1).ok_or(PresaleError::InvalidStage)?;
            } else {
                // still some remaining in this stage; we may have used partial funds and should stop if usd_cents now cannot buy more tokens here
                if usd_cents > 0 {
                    // loop will continue and compute new tokens_units for same stage
                    continue;
                } else {
                    break;
                }
            }
        }

        // update current stage
        state.current_stage = if stage > 5 { 5 } else { stage };

        // if we didn't allocate any tokens (usd_cents couldn't buy even one token at current or later stages) -> error
        if total_minted_raw == 0 {
            return Err(PresaleError::ZeroTokens.into());
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [b"presale_state"],
        bump,
        space = 8 + PresaleState::SIZE
    )]
    pub presale_state: Account<'info, PresaleState>,

    #[account(
        init,
        payer = owner,
        mint::decimals = TOKEN_DECIMALS as u8,
        mint::authority = mint_auth,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = owner,
        seeds = [b"mint_auth"],
        bump,
        space = 8,
    )]
    pub mint_auth: UncheckedAccount<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [b"treasury"],
        bump,
        space = 8,
    )]
    pub treasury: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"presale_state"],
        bump = presale_state.bump
    )]
    pub presale_state: Account<'info, PresaleState>,

    #[account(
        mut,
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"mint_auth"],
        bump
    )]
    pub mint_auth: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

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

#[account]
pub struct PresaleState {
    pub owner: Pubkey,
    pub bump: u8,
    pub mint_bump: u8,
    pub usd_per_sol: u64, // dollars per SOL (whole dollars)
    pub current_stage: u8,

    // stage prices in CENTS (1 == $0.01)
    pub stage_1_price: u64,
    pub stage_2_price: u64,
    pub stage_3_price: u64,
    pub stage_4_price: u64,
    pub stage_5_price: u64,

    // stage caps and sold are RAW token units (token * 10^decimals)
    pub stage_1_cap: u64,
    pub stage_2_cap: u64,
    pub stage_3_cap: u64,
    pub stage_4_cap: u64,
    pub stage_5_cap: u64,

    pub stage_1_sold: u64,
    pub stage_2_sold: u64,
    pub stage_3_sold: u64,
    pub stage_4_sold: u64,
    pub stage_5_sold: u64,
}

impl PresaleState {
    pub const SIZE: usize = 300;

    pub fn current_stage_info(&self) -> Result<(u64, u64)> {
        let (price, cap, sold) = match self.current_stage {
            1 => (self.stage_1_price, self.stage_1_cap, self.stage_1_sold),
            2 => (self.stage_2_price, self.stage_2_cap, self.stage_2_sold),
            3 => (self.stage_3_price, self.stage_3_cap, self.stage_3_sold),
            4 => (self.stage_4_price, self.stage_4_cap, self.stage_4_sold),
            5 => (self.stage_5_price, self.stage_5_cap, self.stage_5_sold),
            _ => return Err(PresaleError::InvalidStage.into()),
        };
        Ok((price, cap - sold))
    }

    // increment_stage is no longer used for allocation (kept to preserve compatibility),
    // state updates are handled inside buy_tokens flexible routine.
    pub fn increment_stage(&mut self, _amount: u64) -> Result<()> {
        Ok(())
    }
}

#[error_code]
pub enum PresaleError {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Invalid oracle price")]
    InvalidOraclePrice,
    #[msg("Zero tokens calculated")]
    ZeroTokens,
    #[msg("Stage cap reached")]
    StageSoldOut,
    #[msg("Invalid stage")]
    InvalidStage,
}
