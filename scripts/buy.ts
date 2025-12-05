// scripts/buy.ts
import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import dotenv from "dotenv";

dotenv.config();

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.Growsol as anchor.Program<any>;

async function buy(amountSol: number) {
  const buyer = provider.wallet.publicKey;

  // PDAs
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

  // Mint address (set in .env by initialize script)
  const mint = new PublicKey(process.env.MINT_ADDRESS!);

  // Derive the buyer's ATA (Anchor will init_if_needed)
  const buyerTokenAccount = await anchor.utils.token.associatedAddress({
    mint,
    owner: buyer,
  });

  console.log("Buyer:", buyer.toBase58());
  console.log("Mint:", mint.toBase58());
  console.log("Buyer ATA:", buyerTokenAccount.toBase58());

  // Convert SOL → lamports (u64)
  const amountLamports = new anchor.BN(Math.floor(amountSol * 1e9));

  const tx = await (program.methods as any)
    .buyTokens(amountLamports)
    .accounts({
      buyer,
      presaleState: presaleStatePda,
      treasury: treasuryPda,
      mintAuth: mintAuthPda,
      mint,
      userTokenAccount: buyerTokenAccount,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log(`\n✅ Bought tokens for ${amountSol} SOL`);
  console.log("Tx:", tx);
}

const amount = Number(process.argv[2]) || 1;
buy(amount).catch(console.error);
