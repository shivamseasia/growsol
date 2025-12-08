import * as anchor from "@coral-xyz/anchor";
import dotenv from "dotenv";
import dayjs from "dayjs";
import { program, presaleStatePda } from "./common";

dotenv.config();

async function setPresaleTimes(startInput: string, endInput: string) {
    // Parse dates
    const start = dayjs(startInput);
    const end = dayjs(endInput);

    if (!start.isValid() || !end.isValid()) {
        console.error("Invalid date format. Use YYYY-MM-DD HH:mm");
        process.exit(1);
    }

    if (end.isBefore(start)) {
        console.error("End time must be after start time");
        process.exit(1);
    }

    const start_ts = start.unix();
    const end_ts = end.unix();

    console.log(`Setting presale start: ${start.format()} (${start_ts})`);
    console.log(`Setting presale end:   ${end.format()} (${end_ts})`);

    const owner = anchor.getProvider().wallet.publicKey;
    const presaleState = presaleStatePda();

    const tx = await (program.methods as any)
        .setPresaleTimes(start_ts, end_ts)
        .accounts({
            owner,
            presaleState,
        })
        .rpc();

    console.log("Transaction successful:", tx);
}

// CLI arguments
const startArg = process.argv[2];
const endArg = process.argv[3];

if (!startArg || !endArg) {
    console.error("Usage: npx ts-node scripts/set_presale_times.ts <start-date> <end-date>");
    console.error("Example: npx ts-node scripts/set_presale_times.ts " + "2025 - 12 - 10 15:00" + "2025 - 12 - 15 23: 59");
    process.exit(1);
}

setPresaleTimes(startArg, endArg).catch(err => {
    console.error("Error:", err);
    process.exit(1);
});
