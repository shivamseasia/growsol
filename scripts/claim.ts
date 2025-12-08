// scripts/claim.ts
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import dotenv from "dotenv";
import { provider, program, userAllocationPdaFor, presaleStatePda, mintAuthPda } from "./common";

dotenv.config();

async function claim(buyerArg?: string) {
  const buyer = buyerArg ? new PublicKey(buyerArg) : provider.wallet.publicKey;

  const presaleState = presaleStatePda();
  const mintAuth = mintAuthPda();
  const mint = new PublicKey(process.env.MINT_ADDRESS!);

  const userAlloc = userAllocationPdaFor(buyer);

  // buyer's ATA (will be init_if_needed by instruction)
  const userAta = await anchor.utils.token.associatedAddress({
    mint,
    owner: buyer,
  });

  console.log("Buyer:", buyer.toBase58());
  console.log("Mint:", mint.toBase58());
  console.log("Buyer ATA:", userAta.toBase58());
  console.log("User allocation PDA:", userAlloc.toBase58());

  const tx = await (program.methods as any)
    .claimTokens()
    .accounts({
      buyer,
      presaleState,
      mintAuth,
      mint,
      userAllocation: userAlloc,
      userTokenAccount: userAta,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("✅ Claim tx:", tx);
}

const arg = process.argv[2]; // optional: pass a buyer pubkey (for admin use, otherwise uses provider wallet)
claim(arg).catch(console.error);
