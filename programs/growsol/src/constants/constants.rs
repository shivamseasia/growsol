use anchor_lang::prelude::*;

#[constant]

pub const TOKEN_DECIMALS: u8 = 9;

/// Raw multiplier = 10^decimals (used to convert token units -> raw units).
pub const TOKEN_BASE: u128 = 1_000_000_000u128; // 10^9

/// Stage token caps (in token units, not raw). Keep these here so you can change supply easily.
pub const STAGE_1_TOKENS: u128 = 150_000_000u128;
pub const STAGE_2_TOKENS: u128 = 200_000_000u128;
pub const STAGE_3_TOKENS: u128 = 200_000_000u128;
pub const STAGE_4_TOKENS: u128 = 225_000_000u128;
pub const STAGE_5_TOKENS: u128 = 225_000_000u128;
/// Number of lamports per SOL.
pub const LAMPORTS_PER_SOL: u128 = 1_000_000_000u128;
pub const USER_ALLOC_SIZE: usize = 32 + 8 + 8 + 1;

pub const PRESALE_SIZE: usize = 32 // owner
        + 1 + 1 + 1 // bumps
        + 8 // usd_per_sol
        + 8 + 8 + 1 // start, end, paused
        + 1 // current_stage
        + (8 * 5) // 5 prices
        + (8 * 5) // 5 caps
        + (8 * 5) // 5 sold
        + 8; // total_allocated

/// SEEDS

pub const MINT_SEED: &[u8] = b"mint_auth";
pub const PRESALE_STATE_SEED: &[u8] = b"presale_state";
pub const TREASURY_SEED: &[u8] = b"treasury";
pub const USER_ALLOC_SEED: &[u8] = b"user_alloc";