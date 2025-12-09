// scripts/common.ts
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import dotenv from "dotenv";
import dayjs from "dayjs";

dotenv.config();

export const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
export const program = anchor.workspace.Growsol as anchor.Program<any>;

export function presaleStatePda(): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from("presale_state")], program.programId)[0];
}

export function presaleConfigPda(): PublicKey {
  return presaleStatePda();
}

export function mintAuthPda(): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from("mint_auth")], program.programId)[0];
}

export function treasuryPda(): PublicKey {
  return PublicKey.findProgramAddressSync([Buffer.from("treasury")], program.programId)[0];
}

/**
 * Derive user_allocation PDA for a given buyer pubkey
 * seeds: ["user_alloc", presale_state.key(), buyer.key()]
 */
export function userAllocationPdaFor(buyer: PublicKey): PublicKey {
  const state = presaleStatePda();
  return PublicKey.findProgramAddressSync(
    [Buffer.from("user_alloc"), state.toBuffer(), buyer.toBuffer()],
    program.programId
  )[0];
}

export function parseDateToUnix(dateStr: string): number {
  const parsed = dayjs(dateStr);
  if (!parsed.isValid()) {
    throw new Error(`Invalid date format: ${dateStr}`);
  }
  return Math.floor(parsed.valueOf() / 1000); // UNIX timestamp in seconds
}

export function solToLamports(sol: number): number {
  return Math.floor(sol * 1_000_000_000);
}
