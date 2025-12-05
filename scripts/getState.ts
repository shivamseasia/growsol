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

  const stagePrices = [0.01, 0.02, 0.03, 0.04, 0.05];
  const stageSold = [
    state.stage1Sold,
    state.stage2Sold,
    state.stage3Sold,
    state.stage4Sold,
    state.stage5Sold,
  ];
  console.log("\nStage | Price (USD) | Tokens Sold | Capital Raised (USD)");
  for (let i = 0; i < 5; i++) {
    const capitalRaised = (stageSold[i] * stagePrices[i]) / 1e9; // assuming token decimals = 9
    console.log(`${i + 1}     | ${stagePrices[i]}       | ${stageSold[i]}        | ${capitalRaised}`);
  }
})();
