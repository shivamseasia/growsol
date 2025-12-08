// scripts/buy.ts
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import dotenv from "dotenv";
import { provider, program, userAllocationPdaFor, presaleStatePda, mintAuthPda, treasuryPda } from "./common";

dotenv.config();

async function buy(amountSol: number) {
  const buyer = provider.wallet.publicKey;

  const presaleState = presaleStatePda();
  const mintAuth = mintAuthPda();
  const treasury = treasuryPda();

  const mint = new PublicKey(process.env.MINT_ADDRESS!);

  // buyer allocation PDA
  const userAlloc = userAllocationPdaFor(buyer);

  console.log("Buyer:", buyer.toBase58());
  console.log("Mint:", mint.toBase58());
  console.log("User allocation PDA:", userAlloc.toBase58());

  // Convert SOL -> lamports (u64)
  const lamports = Math.floor(amountSol * 1e9);
  const amountLamportsBN = new anchor.BN(lamports);

  const tx = await (program.methods as any)
    .buyTokens(amountLamportsBN)
    .accounts({
      buyer,
      presaleState,
      treasury,
      mintAuth,
      mint,
      userAllocation: userAlloc,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log(`\n✅ Bought tokens for ${amountSol} SOL (lamports: ${lamports})`);
  console.log("Tx:", tx);
}

const amount = Number(process.argv[2]) || 1;
buy(amount).catch(console.error);
