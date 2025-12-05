import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import dotenv from "dotenv";

dotenv.config();

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

async function checkBalance(buyerPubkeyString?: string) {
  // Use wallet as default buyer
  const buyer = buyerPubkeyString
    ? new PublicKey(buyerPubkeyString)
    : provider.wallet.publicKey;

  const mint = new PublicKey(process.env.MINT_ADDRESS!);

  // Derive buyer ATA
  const ata = await anchor.utils.token.associatedAddress({
    mint,
    owner: buyer,
  });

  console.log("Buyer:", buyer.toBase58());
  console.log("Mint:", mint.toBase58());
  console.log("Buyer ATA:", ata.toBase58());

  // Fetch SPL token balance
  const tokenAcct = await provider.connection.getTokenAccountBalance(ata);

  console.log("\n====== SPL TOKEN BALANCE ======");
  console.log("Raw Amount:", tokenAcct.value.amount);
  console.log("UI Amount:", tokenAcct.value.uiAmount);
  console.log("Decimals:", tokenAcct.value.decimals);
}

const buyerArg = process.argv[2]; // optional: pass a wallet pubkey
checkBalance(buyerArg).catch(console.error);
