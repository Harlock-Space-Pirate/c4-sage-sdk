import { createHash } from "node:crypto";
import { INSTRUCTIONS } from "../generated/instructions";
import { ACCOUNT_TYPES } from "../generated/accounts";
import { PROGRAM_ID, GAME } from "../generated/constants";
import { starbasePlayerPda, characterPda } from "../pda";

function disc(name: string): string {
  return createHash("sha256").update("global:" + name).digest().subarray(0, 8).toString("hex");
}
function accountDisc(name: string): string {
  return createHash("sha256").update("account:" + name).digest().subarray(0, 8).toString("hex");
}

let failed = 0;
function check(ok: boolean, msg: string) {
  if (!ok) {
    console.error("FAIL", msg);
    failed++;
  } else console.log("ok", msg);
}

check(INSTRUCTIONS.withdraw_atlas.discriminator === disc("withdraw_atlas"), "withdraw_atlas disc");
check(
  INSTRUCTIONS.place_claim_stake_instance.discriminator === disc("place_claim_stake_instance"),
  "place_claim_stake_instance disc",
);
check(ACCOUNT_TYPES.Fleet.discriminator === accountDisc("Fleet"), "Fleet account disc");
check(ACCOUNT_TYPES.StarbasePlayer.discriminator === accountDisc("StarbasePlayer"), "SP account disc");

const a = starbasePlayerPda(GAME, GAME);
const b = starbasePlayerPda(GAME, GAME);
check(a.address === b.address && a.bump === b.bump, "hangar PDA deterministic");
check(characterPda(GAME, GAME).address.length >= 32, "character PDA encodes");
void PROGRAM_ID;

if (failed) process.exit(1);
console.log("smoke ok");
