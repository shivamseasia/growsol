// scripts/withdraw_sol.ts // FOR OWNERS ONLY
import * as anchor from "@coral-xyz/anchor";
import dotenv from "dotenv";
import {
  provider,
  program,
  presaleStatePda,
  treasuryPda,
} from "./common";

dotenv.config();

async function withdraw(solAmount: number) {
  const owner = provider.wallet.publicKey;
  const presaleState = presaleStatePda();
  const treasury = treasuryPda();

  // Convert SOL → lamports
  const lamports = Math.floor(solAmount * 1e9);

  console.log("Owner:", owner.toBase58());
  console.log("Presale State PDA:", presaleState.toBase58());
  console.log("Treasury PDA:", treasury.toBase58());
  console.log(`Withdrawing: ${solAmount} SOL (${lamports} lamports)`);

  const tx = await (program.methods as any)
    .withdrawSol(new anchor.BN(lamports))
    .accounts({
      owner,
      presaleState,
      treasury,
      systemProgram: anchor.web3.SystemProgram.programId, // still required in context
    })
    .rpc();

  console.log("\n✅ Withdraw transaction signature:", tx);
}

const solAmount = Number(process.argv[2]);
if (!solAmount || solAmount <= 0) {
  console.error("❌ Please specify a valid SOL amount, e.g.\nnode scripts/withdraw_sol.ts 1.5");
  process.exit(1);
}

withdraw(solAmount).catch(console.error);
