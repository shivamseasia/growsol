use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::rent::Rent;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("6Mn8wPAxmjeJxJ3YdtMTChA2RTuQF4nFZM6EJ3H34STD");

pub const TOKEN_DECIMALS: u128 = 9; // decimals as u128 for safe math with TOKEN_BASE
pub const TOKEN_BASE: u128 = 1_000_000_000u128; // 10^9, raw multiplier

#[program]
pub mod growsol {
    use super::*;

    /// Initialize the presale state: ladder prices, caps, owner, timing, etc.
    pub fn initialize(
        ctx: Context<Initialize>,
        usd_per_sol: u64, // whole dollars (e.g., 20 => $20)
        presale_start_ts: i64, // unix timestamp
        presale_end_ts: i64,   // unix timestamp
    ) -> Result<()> {
        let state = &mut ctx.accounts.presale_state;

        // bumps
        state.bump = ctx.bumps.presale_state;
        state.mint_bump = ctx.bumps.mint_auth;
        state.treasury_bump = ctx.bumps.treasury;

        // owner & oracle
        state.owner = ctx.accounts.owner.key();
        state.usd_per_sol = usd_per_sol;

        // times & flags
        state.presale_start = presale_start_ts;
        state.presale_end = presale_end_ts;
        state.paused = false;

        // current stage
        state.current_stage = 1;

        // stage prices in cents (1 => $0.01)
        state.stage_1_price = 1;
        state.stage_2_price = 2;
        state.stage_3_price = 3;
        state.stage_4_price = 4;
        state.stage_5_price = 5;

        // stage caps in RAW units (token * 10^decimals)
        state.stage_1_cap = (150_000_000u128.checked_mul(TOKEN_BASE).unwrap()) as u64;
        state.stage_2_cap = (200_000_000u128.checked_mul(TOKEN_BASE).unwrap()) as u64;
        state.stage_3_cap = (200_000_000u128.checked_mul(TOKEN_BASE).unwrap()) as u64;
        state.stage_4_cap = (225_000_000u128.checked_mul(TOKEN_BASE).unwrap()) as u64;
        state.stage_5_cap = (225_000_000u128.checked_mul(TOKEN_BASE).unwrap()) as u64;

        // sold counters
        state.stage_1_sold = 0;
        state.stage_2_sold = 0;
        state.stage_3_sold = 0;
        state.stage_4_sold = 0;
        state.stage_5_sold = 0;

        // total allocated (raw units)
        state.total_allocated = 0;

        emit!(Initialized {
            owner: state.owner,
            start_ts: state.presale_start,
            end_ts: state.presale_end,
        });

        Ok(())
    }

    /// Flexible buy: allocate across stages according to USD value of provided lamports.
    /// Note: This function *allocates* tokens to the user's allocation PDA. It does NOT mint tokens immediately.
    pub fn buy_tokens(ctx: Context<BuyTokens>, sol_amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        let state = &mut ctx.accounts.presale_state;

        // Sale must be active and not paused
        require!(!state.paused, PresaleError::SalePaused);
        require!(clock.unix_timestamp >= state.presale_start, PresaleError::SaleNotStarted);
        require!(clock.unix_timestamp <= state.presale_end, PresaleError::SaleEnded);
        require!(state.usd_per_sol > 0, PresaleError::InvalidOraclePrice);
        require!(sol_amount > 0, PresaleError::ZeroPurchase);

        // transfer SOL from buyer to treasury PDA
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

        // convert lamports -> USD cents
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

        // helper closure to get price cents and remaining raw cap for a stage
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

        // allocate across stages, but DO NOT mint; instead increment sold counters and user allocation PDA
        let mut total_allocated_raw: u128 = 0;
        let mut stage = state.current_stage;

        // ensure we have the user allocation account (PDA) passed in and mutable
        let user_alloc = &mut ctx.accounts.user_allocation;

        while usd_cents > 0 && stage <= 5 {
            let (price_cents, remaining_raw_u64) = stage_price_and_remaining(state, stage)?;
            let remaining_raw: u128 = remaining_raw_u64 as u128;

            if remaining_raw == 0 {
                stage = stage.checked_add(1).ok_or(PresaleError::InvalidStage)?;
                continue;
            }

            // tokens units buyer can buy at this stage: usd_cents / price_cents
            let price_cents_u128 = price_cents as u128;
            let tokens_units = usd_cents.checked_div(price_cents_u128).unwrap_or(0u128);
            if tokens_units == 0 {
                // not enough USD cents to buy a whole token at this stage
                break;
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

            // compute usd_cents_used = allocated_tokens_units * price_cents
            let usd_cents_used = allocated_tokens_units
                .checked_mul(price_cents_u128)
                .ok_or(PresaleError::MathOverflow)?;

            // Update stage sold counters (raw)
            match stage {
                1 => state.stage_1_sold = state
                    .stage_1_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?,
                2 => state.stage_2_sold = state
                    .stage_2_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?,
                3 => state.stage_3_sold = state
                    .stage_3_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?,
                4 => state.stage_4_sold = state
                    .stage_4_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?,
                5 => state.stage_5_sold = state
                    .stage_5_sold
                    .checked_add(to_allocate_raw as u64)
                    .ok_or(PresaleError::MathOverflow)?,
                _ => return Err(PresaleError::InvalidStage.into()),
            }

            total_allocated_raw = total_allocated_raw
                .checked_add(to_allocate_raw)
                .ok_or(PresaleError::MathOverflow)?;

            // subtract used USD cents and continue
            usd_cents = usd_cents
                .checked_sub(usd_cents_used)
                .ok_or(PresaleError::MathOverflow)?;

            // advance stage if exhausted
            let (_, rem_after) = stage_price_and_remaining(state, stage)?;
            if rem_after == 0 {
                stage = stage.checked_add(1).ok_or(PresaleError::InvalidStage)?;
            } else {
                // still tokens left in this stage — loop may continue if usd_cents still allows more purchase
                continue;
            }
        } // end while

        // update current stage
        state.current_stage = if stage > 5 { 5 } else { stage };

        require!(total_allocated_raw > 0u128, PresaleError::ZeroTokens);

        // update state totals
        state.total_allocated = state
            .total_allocated
            .checked_add(total_allocated_raw as u64)
            .ok_or(PresaleError::MathOverflow)?;

        // update or create user allocation account
        // user_allocation account is a PDA unique per buyer
        user_alloc.buyer = ctx.accounts.buyer.key();
        user_alloc.allocated_raw = user_alloc
            .allocated_raw
            .checked_add(total_allocated_raw as u64)
            .ok_or(PresaleError::MathOverflow)?;
        // claimed remains whatever it was

        emit!(TokensAllocated {
            buyer: user_alloc.buyer,
            allocated_raw: total_allocated_raw as u64,
        });

        Ok(())
    }

    /// Claim tokens: mints all unclaimed tokens allocated to the user into their ATA.
    /// This supports delayed distribution / vesting if you'd like to gate claims externally.
    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        let state = &mut ctx.accounts.presale_state;
        let user_alloc = &mut ctx.accounts.user_allocation;

        require!(user_alloc.buyer == ctx.accounts.buyer.key(), PresaleError::Unauthorized);

        // compute claimable raw = allocated_raw - claimed_raw
        let allocated = user_alloc.allocated_raw as u128;
        let claimed = user_alloc.claimed_raw as u128;
        require!(allocated > claimed, PresaleError::NothingToClaim);

        let to_claim_raw_u128 = allocated.checked_sub(claimed).ok_or(PresaleError::MathOverflow)?;
        let to_claim_raw_u64 = to_claim_raw_u128
            .try_into()
            .map_err(|_| PresaleError::MathOverflow)?;

        // Mint tokens to buyer ATA using mint_auth PDA as signer
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

    /// Owner withdraw SOL from treasury PDA to the owner's wallet
    pub fn withdraw_sol(ctx: Context<WithdrawSol>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.presale_state;
        require!(ctx.accounts.owner.key() == state.owner, PresaleError::Unauthorized);

        // Transfer lamports from treasury to owner
        // treasury is a system account; to move lamports we need to invoke system program with signer seeds (treasury PDA is not a signer normally,
        // but system_program::transfer uses from account as signer — we actually must use invoke_signed to debit the PDA).
        // However, Anchor's system_program::transfer cannot sign for a PDA; instead we use invoke_signed.
        let treasury_info = ctx.accounts.treasury.to_account_info();
        let owner_info = ctx.accounts.owner.to_account_info();

        let seeds = &[b"treasury".as_ref(), &[state.treasury_bump][..]];
        let signer_seeds = &[&seeds[..]];

        anchor_lang::solana_program::program::invoke_signed(
            &anchor_lang::solana_program::system_instruction::transfer(
                treasury_info.key,
                owner_info.key,
                amount,
            ),
            &[
                treasury_info.clone(),
                owner_info.clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            signer_seeds,
        )?;

        emit!(WithdrawnSol {
            owner: state.owner,
            amount,
        });

        Ok(())
    }

    /// Owner withdraw tokens from mint to an owner's ATA (for leftover tokens or team allocation).
    pub fn withdraw_token(ctx: Context<WithdrawToken>, amount_raw: u64) -> Result<()> {
        let state = &ctx.accounts.presale_state;
        require!(ctx.accounts.owner.key() == state.owner, PresaleError::Unauthorized);

        let mint_auth_seeds = &[b"mint_auth".as_ref(), &[state.mint_bump][..]];
        let signer_seeds = &[&mint_auth_seeds[..]];

        // Mint to owner's token account (owner must provide their ATA)
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.owner_token_account.to_account_info(),
                    authority: ctx.accounts.mint_auth.to_account_info(),
                },
                signer_seeds,
            ),
            amount_raw,
        )?;

        emit!(WithdrawnToken {
            owner: state.owner,
            amount_raw,
        });

        Ok(())
    }

    /// Set presale start/end timestamps (owner only)
    pub fn set_presale_times(ctx: Context<AdminSetTimes>, start_ts: i64, end_ts: i64) -> Result<()> {
        let state: &mut Account<'_, PresaleState> = &mut ctx.accounts.presale_state;
        require!(ctx.accounts.owner.key() == state.owner, PresaleError::Unauthorized);

        state.presale_start = start_ts;
        state.presale_end = end_ts;

        emit!(PresaleTimesUpdated {
            start_ts,
            end_ts,
        });

        Ok(())
    }

    /// Pause sale
    pub fn pause_sale(ctx: Context<AdminToggleSale>) -> Result<()> {
        let state = &mut ctx.accounts.presale_state;
        require!(ctx.accounts.owner.key() == state.owner, PresaleError::Unauthorized);
        state.paused = true;
        emit!(SalePaused { owner: state.owner });
        Ok(())
    }

    /// Resume sale
    pub fn resume_sale(ctx: Context<AdminToggleSale>) -> Result<()> {
        let state = &mut ctx.accounts.presale_state;
        require!(ctx.accounts.owner.key() == state.owner, PresaleError::Unauthorized);
        state.paused = false;
        emit!(SaleResumed { owner: state.owner });
        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(usd_per_sol: u64, presale_start_ts: i64, presale_end_ts: i64)]
pub struct Initialize<'info> {
    /// Owner/initializer
    #[account(mut)]
    pub owner: Signer<'info>,

    /// presale state PDA
    #[account(
        init,
        payer = owner,
        seeds = [b"presale_state"],
        bump,
        space = 8 + PresaleState::SIZE
    )]
    pub presale_state: Account<'info, PresaleState>,

    /// token mint
    #[account(
        init,
        payer = owner,
        mint::decimals = 9,
        mint::authority = mint_auth,
    )]
    pub mint: Account<'info, Mint>,

    /// mint authority PDA
    #[account(
        init,
        seeds = [b"mint_auth"],
        bump,
        payer = owner,
        space = 8,
    )]
    pub mint_auth: UncheckedAccount<'info>,

    /// treasury PDA (holds SOL)
    #[account(
        init,
        seeds = [b"treasury"],
        bump,
        payer = owner,
        space = 8,
    )]
    pub treasury: UncheckedAccount<'info>,

    /// programs
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

    /// treasury PDA (SOL receiver)
    #[account(mut, seeds = [b"treasury"], bump)]
    pub treasury: UncheckedAccount<'info>,

    /// mint authority PDA (not signer)
    #[account(mut, seeds = [b"mint_auth"], bump)]
    pub mint_auth: UncheckedAccount<'info>,

    /// mint (already initialized)
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// per-user allocation PDA (created by buyer if needed)
    /// seeds: ["user_alloc", presale_state.key(), buyer.key()]
    #[account(
        init_if_needed,
        payer = buyer,
        seeds = [b"user_alloc", presale_state.key().as_ref(), buyer.key().as_ref()],
        bump,
        space = 8 + UserAllocation::SIZE
    )]
    pub user_allocation: Account<'info, UserAllocation>,

    /// system / token programs
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(mut, seeds = [b"presale_state"], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,

    /// mint auth PDA
    #[account(mut, seeds = [b"mint_auth"], bump)]
    pub mint_auth: UncheckedAccount<'info>,

    /// mint
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// user allocation PDA
    #[account(mut, seeds = [b"user_alloc", presale_state.key().as_ref(), buyer.key().as_ref()], bump)]
    pub user_allocation: Account<'info, UserAllocation>,

    /// buyer's ATA (will be created if needed by associated_token program)
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

#[derive(Accounts)]
pub struct WithdrawSol<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    /// presale state for auth
    #[account(seeds = [b"presale_state"], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,

    /// treasury PDA
    #[account(mut, seeds = [b"treasury"], bump)]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(seeds = [b"presale_state"], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,

    #[account(mut, seeds = [b"mint_auth"], bump)]
    pub mint_auth: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// owner's ATA to receive tokens
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = mint,
        associated_token::authority = owner,
    )]
    pub owner_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AdminSetTimes<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, seeds = [b"presale_state"], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,
}

#[derive(Accounts)]
pub struct AdminToggleSale<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut, seeds = [b"presale_state"], bump = presale_state.bump)]
    pub presale_state: Account<'info, PresaleState>,
}

//
// Data structs stored on-chain
//

/// Per-user allocation record (raw token units)
#[account]
pub struct UserAllocation {
    pub buyer: Pubkey,
    pub allocated_raw: u64, // total allocated (raw units)
    pub claimed_raw: u64,   // total claimed (raw units)
    // future: add refunded flag, vesting schedule fields, etc.
}

impl UserAllocation {
    pub const SIZE: usize = 32 + 8 + 8; // buyer + allocated + claimed
}

/// Presale state account
#[account]
pub struct PresaleState {
    pub owner: Pubkey,
    pub bump: u8,
    pub mint_bump: u8,
    pub treasury_bump: u8,

    pub usd_per_sol: u64, // whole dollars, e.g. 20 => $20

    pub presale_start: i64,
    pub presale_end: i64,
    pub paused: bool,

    pub current_stage: u8,

    // prices in cents
    pub stage_1_price: u64,
    pub stage_2_price: u64,
    pub stage_3_price: u64,
    pub stage_4_price: u64,
    pub stage_5_price: u64,

    // caps & sold in raw units
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

    // totals
    pub total_allocated: u64, // raw units allocated across users
}

impl PresaleState {
    // estimate size: adjust if you add fields
    pub const SIZE: usize = 32  // owner
        + 1 + 1 + 1               // bumps
        + 8                       // usd_per_sol
        + 8 + 8 + 1               // times + paused
        + 1                       // current_stage
        + (8 * 5) * 2             // 5 prices + 5 caps (u64 each) (actually doubled; keep safe)
        + (8 * 5) * 2             // 5 sold + total_allocated (safety)
        ;
}

//
// Events
//
#[event]
pub struct Initialized {
    pub owner: Pubkey,
    pub start_ts: i64,
    pub end_ts: i64,
}

#[event]
pub struct TokensAllocated {
    pub buyer: Pubkey,
    pub allocated_raw: u64,
}

#[event]
pub struct TokensClaimed {
    pub buyer: Pubkey,
    pub claimed_raw: u64,
}

#[event]
pub struct WithdrawnSol {
    pub owner: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawnToken {
    pub owner: Pubkey,
    pub amount_raw: u64,
}

#[event]
pub struct PresaleTimesUpdated {
    pub start_ts: i64,
    pub end_ts: i64,
}

#[event]
pub struct SalePaused {
    pub owner: Pubkey,
}

#[event]
pub struct SaleResumed {
    pub owner: Pubkey,
}

//
// Errors
//
#[error_code]
pub enum PresaleError {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Invalid oracle price")]
    InvalidOraclePrice,
    #[msg("Zero purchase amount")]
    ZeroPurchase,
    #[msg("Zero tokens allocated")]
    ZeroTokens,
    #[msg("Sale not started")]
    SaleNotStarted,
    #[msg("Sale ended")]
    SaleEnded,
    #[msg("Sale is paused")]
    SalePaused,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid stage")]
    InvalidStage,
    #[msg("Nothing to claim")]
    NothingToClaim,
    #[msg("Buyer not authorized for this allocation")]
    UnauthorizedBuyer,
}
