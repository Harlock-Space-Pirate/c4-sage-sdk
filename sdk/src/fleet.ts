import { ACCOUNT_TYPES } from "./generated/accounts";
import { encodePk } from "./bytes";

/**
 * Fleet readout from live account offsets.
 * TOTAL HULL / SHIELD are NOT on the fleet — client sums Game ship configs.
 */
export type FleetReadout = {
  fleetLabel: string;
  ownerProfile: string;
  subProfile: string;
  requiredCrew: number;
  passengerCapacity: number;
  respawnTimeWithoutFee: number;
  crewCount: number;
  rentedCrew: number;
  ap: number;
  fuelId: number;
  fuelAmount: bigint;
  ammoId: number;
  ammoAmount: bigint;
};

function off(name: string): number {
  const f = ACCOUNT_TYPES.Fleet.fields.find((x) => x.name === name);
  if (!f) throw new Error(`Fleet field ${name} missing`);
  return f.offset;
}

export function decodeFleetReadout(data: Buffer): FleetReadout {
  const labelAt = off("fleetLabel");
  return {
    fleetLabel: data.subarray(labelAt, labelAt + 64).toString("utf8").replace(/\0+$/, ""),
    ownerProfile: encodePk(data.subarray(off("ownerProfile"), off("ownerProfile") + 32)),
    subProfile: encodePk(data.subarray(off("subProfile"), off("subProfile") + 32)),
    requiredCrew: data.readUInt16LE(off("requiredCrew")),
    passengerCapacity: data.readUInt16LE(off("passengerCapacity")),
    respawnTimeWithoutFee: data.readUInt16LE(off("respawnTimeWithoutFee")),
    crewCount: data.readUInt16LE(off("crewCount")),
    rentedCrew: data.readUInt16LE(off("rentedCrew")),
    ap: data.readUInt32LE(off("ap")),
    fuelId: data.readUInt16LE(off("fuelTank.cargoId")),
    fuelAmount: data.readBigUInt64LE(off("fuelTank.amount")),
    ammoId: data.readUInt16LE(off("ammoBank.cargoId")),
    ammoAmount: data.readBigUInt64LE(off("ammoBank.amount")),
  };
}
