// scripts/initialize.ts
import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import dotenv from "dotenv";

dotenv.config();

// Set up Anchor provider
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.Growsol as anchor.Program<any>;

async function initialize() {
  const owner = provider.wallet.publicKey;

  // Derive PDAs
  const [presaleStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("presale_state")],
    program.programId
  );

  const [mintAuthPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("mint_auth")],
    program.programId
  );

  const [treasuryPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("treasury")],
    program.programId
  );

  // Create new token mint
  const mint = Keypair.generate();

  console.log("Owner wallet:", owner.toBase58());
  console.log("PresaleState PDA:", presaleStatePda.toBase58());
  console.log("Mint:", mint.publicKey.toBase58());
  console.log("Mint Auth PDA:", mintAuthPda.toBase58());
  console.log("Treasury PDA:", treasuryPda.toBase58());

  // Call initialize instruction
  const usdPerSol = new anchor.BN(20); // Example: 20 USD per SOL

  const tx = await (program.methods as any)
    .initialize(usdPerSol)
    .accounts({
      owner,                    // provider wallet signs
      presaleState: presaleStatePda,
      mint: mint.publicKey,
      mintAuth: mintAuthPda,
      treasury: treasuryPda,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
      rent: anchor.web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([mint]) // only sign with the new mint keypair
    .rpc();

  console.log("Initialization transaction signature:", tx);
  console.log("\nIMPORTANT: save this mint address to your .env as MINT_ADDRESS:");
  console.log("MINT_ADDRESS=" + mint.publicKey.toBase58());
}

initialize().catch(console.error);
