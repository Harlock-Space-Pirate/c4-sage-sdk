import { ACCOUNT_TYPES } from "./generated/accounts";
import { jsonRpc, getAccountData, type Rpc } from "./rpc";
import { decodeAccount, identifyAccount, fieldMap } from "./decode";
import { decodeFleetReadout } from "./fleet";
import { decodeGameHeader, decodeItemTable, decodeItemCategories } from "./game";
import { buildIx, type NamedAccounts, type BuiltIx } from "./ix";
import { resolveConfig, type ClusterConfig, type ClusterOverrides } from "./config";
import { signAndSend, simulateIxs, type Signer, type SimResult } from "./tx";

export class C4SageClient {
  readonly config: ClusterConfig;
  readonly rpc: Rpc;
  readonly programId: string;
  readonly game: string;

  constructor(opts?: ClusterOverrides) {
    this.config = resolveConfig(opts);
    this.rpc = jsonRpc(this.config.rpcUrl);
    this.programId = this.config.programId;
    this.game = this.config.game;
  }

  async fetchRaw(pubkey: string): Promise<Buffer | null> {
    return getAccountData(this.rpc, pubkey);
  }

  async fetchDecoded(pubkey: string) {
    const data = await this.fetchRaw(pubkey);
    if (!data) return null;
    const decoded = decodeAccount(data, identifyAccount(data));
    return { pubkey, data, ...decoded, map: fieldMap(decoded) };
  }

  async fetchFleet(pubkey: string) {
    const data = await this.fetchRaw(pubkey);
    if (!data) return null;
    const t = ACCOUNT_TYPES.Fleet;
    if (data.subarray(0, 8).toString("hex") !== t.discriminator) throw new Error("not a Fleet account");
    return { readout: decodeFleetReadout(data), decoded: decodeAccount(data, t), data };
  }

  async fetchGame(pubkey = this.game) {
    const data = await this.fetchRaw(pubkey);
    if (!data) return null;
    return {
      header: decodeGameHeader(data),
      items: decodeItemTable(data, 16),
      categories: decodeItemCategories(data),
      bytes: data.length,
    };
  }

  buildIx(
    name: string,
    accounts: NamedAccounts,
    args?: Record<string, number | bigint | Uint8Array> | Uint8Array,
  ): BuiltIx {
    return buildIx(name, accounts, args, this.programId);
  }

  /** Simulate. Does not land a transaction. Signatures may be dummy if sigVerify is off. */
  async simulate(
    ixs: BuiltIx[],
    feePayer: string,
    signers: Signer[] = [],
  ): Promise<SimResult> {
    return simulateIxs(this.rpc, ixs, feePayer, signers);
  }

  /** Sign with caller-supplied signers and send. The SDK does not store keys. */
  async signAndSend(ixs: BuiltIx[], signers: Signer[]): Promise<string> {
    return signAndSend(this.rpc, ixs, signers);
  }
}
