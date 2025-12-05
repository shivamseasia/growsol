use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::rent::Rent;
use anchor_lang::system_program;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("6Mn8wPAxmjeJxJ3YdtMTChA2RTuQF4nFZM6EJ3H34STD");

pub const TOKEN_DECIMALS: u8 = 9;

#[program]
pub mod growsol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, usd_per_sol: u64) -> Result<()> {
        let state = &mut ctx.accounts.presale_state;

        // Save bumps for PDAs
        state.bump = ctx.bumps.presale_state;
        state.mint_bump = ctx.bumps.mint_auth;

        state.owner = ctx.accounts.owner.key();
        state.usd_per_sol = usd_per_sol;
        state.current_stage = 1;

        // Example stage data
        state.stage_1_price = 1;
        state.stage_2_price = 2;
        state.stage_3_price = 3;
        state.stage_4_price = 4;
        state.stage_5_price = 5;

        state.stage_1_cap = 150_000_000 * 1_000_000_000;
        state.stage_2_cap = 200_000_000 * 1_000_000_000;
        state.stage_3_cap = 200_000_000 * 1_000_000_000;
        state.stage_4_cap = 225_000_000 * 1_000_000_000;
        state.stage_5_cap = 225_000_000 * 1_000_000_000;

        state.stage_1_sold = 0;
        state.stage_2_sold = 0;
        state.stage_3_sold = 0;
        state.stage_4_sold = 0;
        state.stage_5_sold = 0;

        Ok(())
    }

    pub fn buy_tokens(ctx: Context<BuyTokens>, sol_amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.presale_state;
        require!(state.usd_per_sol > 0, PresaleError::InvalidOraclePrice);

        let usd_value = sol_amount
            .checked_mul(state.usd_per_sol)
            .ok_or(PresaleError::MathOverflow)?;

        let (price, remaining_cap) = state.current_stage_info()?;
        let tokens_to_mint = usd_value
            .checked_div(price as u64)
            .ok_or(PresaleError::ZeroTokens)?;

        require!(tokens_to_mint <= remaining_cap, PresaleError::StageSoldOut);

        // Transfer SOL to treasury PDA
        let treasury_bump = ctx.bumps.treasury;
        let treasury_seeds = &[b"treasury".as_ref(), &[treasury_bump][..]];
        let treasury_signer = &[&treasury_seeds[..]];

        system_program::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.treasury.to_account_info(),
                },
                treasury_signer,
            ),
            sol_amount,
        )?;

        // Mint tokens to buyer
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
            tokens_to_mint,
        )?;

        state.increment_stage(tokens_to_mint)?;

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
        mint::decimals = TOKEN_DECIMALS,
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
    pub usd_per_sol: u64,
    pub current_stage: u8,
    pub stage_1_price: u64,
    pub stage_2_price: u64,
    pub stage_3_price: u64,
    pub stage_4_price: u64,
    pub stage_5_price: u64,
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

    pub fn increment_stage(&mut self, amount: u64) -> Result<()> {
        match self.current_stage {
            1 => {
                self.stage_1_sold += amount;
                if self.stage_1_sold >= self.stage_1_cap {
                    self.current_stage = 2;
                }
            }
            2 => {
                self.stage_2_sold += amount;
                if self.stage_2_sold >= self.stage_2_cap {
                    self.current_stage = 3;
                }
            }
            3 => {
                self.stage_3_sold += amount;
                if self.stage_3_sold >= self.stage_3_cap {
                    self.current_stage = 4;
                }
            }
            4 => {
                self.stage_4_sold += amount;
                if self.stage_4_sold >= self.stage_4_cap {
                    self.current_stage = 5;
                }
            }
            5 => {
                self.stage_5_sold += amount;
            }
            _ => return Err(PresaleError::InvalidStage.into()),
        }
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
