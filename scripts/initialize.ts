// scripts/initialize.ts
import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import dotenv from "dotenv";
import { program, provider, presaleConfigPda, presaleStatePda } from "./common";

dotenv.config();

async function initialize() {
  const owner = provider.wallet.publicKey;

  const presaleConfig = presaleConfigPda();
  const presaleState = presaleStatePda();

  const [mintAuthPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("mint_auth")],
    program.programId
  );

  const [treasuryPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury")],
    program.programId
  );

  const mint = Keypair.generate();

  console.log("Owner wallet:", owner.toBase58());
  console.log("PresaleConfig PDA:", presaleConfig.toBase58());
  console.log("PresaleState PDA:", presaleState.toBase58());
  console.log("Mint:", mint.publicKey.toBase58());
  console.log("Mint Auth PDA:", mintAuthPda.toBase58());
  console.log("Treasury PDA:", treasuryPda.toBase58());

  const usdPerSolNumber = Number(process.argv[2]) || 20;

  const now = Math.floor(Date.now() / 1000);

  const startTsBN = new anchor.BN(Number(process.argv[3]) || now);
  const endTsBN = new anchor.BN(Number(process.argv[4]) || now + 7 * 24 * 3600);

  console.log(
    `Using usd_per_sol=${usdPerSolNumber}, start=${startTsBN.toString()}, end=${endTsBN.toString()}`
  );

  const tx = await (program.methods as any)
    .initialize(
      new anchor.BN(usdPerSolNumber),
      startTsBN,
      endTsBN
    )
    .accounts({
      owner,
      presaleConfig,
      presaleState,
      mint: mint.publicKey,
      mintAuth: mintAuthPda,
      treasury: treasuryPda,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([mint])
    .rpc();

  console.log("Initialization transaction signature:", tx);
  console.log("\nIMPORTANT: save this mint address to your .env as MINT_ADDRESS:");
  console.log("MINT_ADDRESS=" + mint.publicKey.toBase58());

  console.log("\nPresale Stage Setup:");
  console.log("Stage | Price (USD) | Cap (Tokens)");
  console.log("1     | 0.01        | 150,000,000");
  console.log("2     | 0.02        | 200,000,000");
  console.log("3     | 0.03        | 200,000,000");
  console.log("4     | 0.04        | 225,000,000");
  console.log("5     | 0.05        | 225,000,000");
}

initialize().catch(console.error);
