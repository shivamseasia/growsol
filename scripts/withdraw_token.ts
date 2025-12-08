// scripts/withdraw_token.ts // FOR OWNERS ONLY
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import dotenv from "dotenv";
import { provider, program, presaleStatePda, mintAuthPda } from "./common";

dotenv.config();

async function withdrawToken(amountRaw: number) {
  const owner = provider.wallet.publicKey;
  const presaleState = presaleStatePda();
  const mintAuth = mintAuthPda();
  const mint = new PublicKey(process.env.MINT_ADDRESS!);

  // Owner ATA for mint (will be created if missing)
  const ownerAta = await anchor.utils.token.associatedAddress({ mint, owner });

  // Presale vault (ATA of presale_state PDA holding tokens)
  const presaleTokenAccount = await anchor.utils.token.associatedAddress({
    mint,
    owner: presaleState, // presale_state PDA owns the vault
  });

  console.log("Owner:", owner.toBase58());
  console.log("Mint:", mint.toBase58());
  console.log("Owner ATA:", ownerAta.toBase58());
  console.log("Presale token ATA:", presaleTokenAccount.toBase58());
  console.log("Amount (raw units):", amountRaw);

  const tx = await (program.methods as any)
    .withdrawToken(new anchor.BN(amountRaw))
    .accounts({
      owner,
      presaleState,
      presaleTokenAccount,       // <-- must provide now
      mintAuth,
      mint,
      ownerTokenAccount: ownerAta,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("Withdraw token tx:", tx);
}

const amtRaw = Number(process.argv[2]) || 0;
if (!amtRaw || amtRaw <= 0) {
  console.error("Usage: node scripts/withdraw_token.ts <amount_raw>");
  process.exit(1);
}
withdrawToken(amtRaw).catch(console.error);
