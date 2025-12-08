// scripts/pause_resume.ts // FOR OWNERS ONLY
import * as anchor from "@coral-xyz/anchor";
import dotenv from "dotenv";
import { program, presaleStatePda } from "./common";

dotenv.config();

async function pauseOrResume(cmd: "pause" | "resume") {
  const owner = anchor.getProvider().wallet.publicKey;
  const presaleState = presaleStatePda();

  if (cmd === "pause") {
    const tx = await (program.methods as any)
      .pauseSale()
      .accounts({
        owner,
        presaleState,
      })
      .rpc();
    console.log("Paused. Tx:", tx);
  } else {
    const tx = await (program.methods as any)
      .resumeSale()
      .accounts({
        owner,
        presaleState,
      })
      .rpc();
    console.log("Resumed. Tx:", tx);
  }
}

const cmd = process.argv[2] as "pause" | "resume";
if (!cmd || (cmd !== "pause" && cmd !== "resume")) {
  console.error("Usage: node scripts/pause_resume.ts <pause|resume>");
  process.exit(1);
}
pauseOrResume(cmd).catch(console.error);
