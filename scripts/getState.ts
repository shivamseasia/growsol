import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import dotenv from "dotenv";

dotenv.config();
anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.Growsol as anchor.Program;

(async () => {
  const [presaleStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("presale_state")],
    program.programId
  );

  const state = await (program.account as any).presaleState.fetch(presaleStatePda);

  console.log("📊 GrowSol Presale State:");
  console.log("Current Stage:", state.currentStage.toString());
  console.log("USD Per SOL:", state.usdPerSol.toString());
  console.log("Stage 1 Sold:", state.stage1Sold.toString());
  console.log("Stage 2 Sold:", state.stage2Sold.toString());
  console.log("Stage 3 Sold:", state.stage3Sold.toString());
  console.log("Stage 4 Sold:", state.stage4Sold.toString());
  console.log("Stage 5 Sold:", state.stage5Sold.toString());
})();

