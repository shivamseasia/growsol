import * as anchor from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
} from "@solana/web3.js";

import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";

import idl from "../target/idl/growsol.json";

describe("GrowSol Presale", () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;

  const program = new anchor.Program(idl as anchor.Idl, provider);

  // Owner = test wallet
  const owner = wallet.payer;

  // Mint keypair (signer in initialize)
  const mintKeypair = Keypair.generate();

  let presaleStatePda: PublicKey;
  let treasuryPda: PublicKey;
  let mintAuthPda: PublicKey;
  let presaleTokenAta: PublicKey;

  const buyer = Keypair.generate();

  before("Airdrop SOL to participants", async () => {
    // Airdrop to owner
    await provider.connection.requestAirdrop(owner.publicKey, 5 * LAMPORTS_PER_SOL);
    // Airdrop to buyer
    await provider.connection.requestAirdrop(buyer.publicKey, 5 * LAMPORTS_PER_SOL);
  });

  it("Derives all PDAs", async () => {
    [presaleStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("presale_state")],
      program.programId
    );

    [treasuryPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("treasury")],
      program.programId
    );

    [mintAuthPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint_auth")],
      program.programId
    );

    presaleTokenAta = getAssociatedTokenAddressSync(
      mintKeypair.publicKey,
      presaleStatePda,
      true
    );

    console.log("📌 Presale State PDA:", presaleStatePda.toBase58());
    console.log("📌 Treasury PDA:", treasuryPda.toBase58());
    console.log("📌 Mint Auth PDA:", mintAuthPda.toBase58());
    console.log("📌 Presale Token ATA:", presaleTokenAta.toBase58());
  });

  it("Initializes the presale", async () => {
    const now = Math.floor(Date.now() / 1000);

    await program.methods
      .initialize(
        new anchor.BN(120),          // usd_per_sol
        new anchor.BN(now - 10),     // start
        new anchor.BN(now + 5000)    // end
      )
      .accounts({
        owner: owner.publicKey,
        presaleState: presaleStatePda,
        mint: mintKeypair.publicKey,
        mintAuth: mintAuthPda,
        treasury: treasuryPda,
        presaleTokenAccount: presaleTokenAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([owner, mintKeypair])
      .rpc();

    console.log("✅ Initialize complete");
  });

  it("Owner sets presale times", async () => {
    const newStart = Math.floor(Date.now() / 1000);
    const newEnd = newStart + 8000;

    await program.methods
      .adminSetTimes(new anchor.BN(newStart), new anchor.BN(newEnd))
      .accounts({
        owner: owner.publicKey,
        presaleState: presaleStatePda,
      })
      .signers([owner])
      .rpc();

    console.log("✅ admin_set_times updated");
  });

  let buyerUserAllocPda: PublicKey;

  it("Buyer purchases tokens", async () => {
    [buyerUserAllocPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("user_alloc"),
        presaleStatePda.toBuffer(),
        buyer.publicKey.toBuffer(),
      ],
      program.programId
    );

    await program.methods
      .buyTokens(new anchor.BN(1 * LAMPORTS_PER_SOL))
      .accounts({
        buyer: buyer.publicKey,
        presaleState: presaleStatePda,
        treasury: treasuryPda,
        mintAuth: mintAuthPda,
        mint: mintKeypair.publicKey,
        userAllocation: buyerUserAllocPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    console.log("✅ buy_tokens executed for buyer:", buyer.publicKey.toBase58());
  });

  it("Buyer claims tokens", async () => {
    const buyerAta = getAssociatedTokenAddressSync(
      mintKeypair.publicKey,
      buyer.publicKey,
      false
    );

    await program.methods
      .claimTokens()
      .accounts({
        buyer: buyer.publicKey,
        presaleState: presaleStatePda,
        mintAuth: mintAuthPda,
        mint: mintKeypair.publicKey,
        userAllocation: buyerUserAllocPda,
        userTokenAccount: buyerAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    console.log("✅ claim_tokens successful");
  });
});
