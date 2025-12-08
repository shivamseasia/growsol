// scripts/check_balance.ts
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import dotenv from "dotenv";
import { provider } from "./common";

dotenv.config();
const p = provider;
anchor.setProvider(p);

async function checkBalance(buyerPubkeyString?: string) {
  const buyer = buyerPubkeyString ? new PublicKey(buyerPubkeyString) : p.wallet.publicKey;
  const mint = new PublicKey(process.env.MINT_ADDRESS!);

  const ata = await anchor.utils.token.associatedAddress({
    mint,
    owner: buyer,
  });

  console.log("Buyer:", buyer.toBase58());
  console.log("Mint:", mint.toBase58());
  console.log("Buyer ATA:", ata.toBase58());

  try {
    const tokenAcct = await p.connection.getTokenAccountBalance(ata);
    console.log("\n====== SPL TOKEN BALANCE ======");
    console.log("Raw Amount:", tokenAcct.value.amount);
    // tokenAcct.value.uiAmountString sometimes undefined in older web3 versions -> use uiAmount if present
    console.log("UI Amount:", tokenAcct.value.uiAmountString ?? tokenAcct.value.uiAmount);
    console.log("Decimals:", tokenAcct.value.decimals);
  } catch (err) {
    console.log("Account not found or zero balance.");
  }
}

const buyerArg = process.argv[2];
checkBalance(buyerArg).catch(console.error);
