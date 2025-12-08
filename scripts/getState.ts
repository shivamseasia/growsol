// scripts/getState.ts
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import dotenv from "dotenv";
import { presaleStatePda, program } from "./common";

dotenv.config();
anchor.setProvider(anchor.AnchorProvider.env());

(async () => {
  const presaleState = presaleStatePda();

  const state = await (program.account as any).presaleState.fetch(presaleState);

  // helper to read both snake_case and camelCase returned fields
  const get = (o: any, keys: string[]) => {
    for (const k of keys) if (o[k] !== undefined) return o[k];
    return undefined;
  };

  const currentStage = get(state, ["current_stage", "currentStage"]);
  const usdPerSol = get(state, ["usd_per_sol", "usdPerSol"]);
  const stage1Sold = get(state, ["stage_1_sold", "stage1Sold"]);
  const stage2Sold = get(state, ["stage_2_sold", "stage2Sold"]);
  const stage3Sold = get(state, ["stage_3_sold", "stage3Sold"]);
  const stage4Sold = get(state, ["stage_4_sold", "stage4Sold"]);
  const stage5Sold = get(state, ["stage_5_sold", "stage5Sold"]);

  console.log("📊 GrowSol Presale State:");
  console.log("Current Stage:", currentStage?.toString?.() ?? currentStage);
  console.log("USD Per SOL:", usdPerSol?.toString?.() ?? usdPerSol);
  console.log("Stage 1 Sold:", stage1Sold?.toString?.() ?? stage1Sold);
  console.log("Stage 2 Sold:", stage2Sold?.toString?.() ?? stage2Sold);
  console.log("Stage 3 Sold:", stage3Sold?.toString?.() ?? stage3Sold);
  console.log("Stage 4 Sold:", stage4Sold?.toString?.() ?? stage4Sold);
  console.log("Stage 5 Sold:", stage5Sold?.toString?.() ?? stage5Sold);

  // Print human readable table (assumes the stage prices are the known ladder)
  const stagePrices = [0.01, 0.02, 0.03, 0.04, 0.05];
  const stageSoldArr = [
    Number(stage1Sold ?? 0),
    Number(stage2Sold ?? 0),
    Number(stage3Sold ?? 0),
    Number(stage4Sold ?? 0),
    Number(stage5Sold ?? 0),
  ];

  console.log("\nStage | Price (USD) | Tokens Sold (raw) | Capital Raised (USD)");
  for (let i = 0; i < 5; i++) {
    // stageSoldArr is raw token units (multiplied by 1e9). Convert to token units:
    const tokensSoldUnits = stageSoldArr[i] / 1e9;
    const capitalRaised = tokensSoldUnits * stagePrices[i];
    console.log(
      `${i + 1}     | ${stagePrices[i]}       | ${tokensSoldUnits}        | ${capitalRaised}`
    );
  }
})();
