export type Rpc = (method: string, params: unknown[]) => Promise<unknown>;

export function jsonRpc(url: string): Rpc {
  return async (method, params) => {
    const res = await fetch(url, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
    });
    const body = (await res.json()) as { result?: unknown; error?: { message: string } };
    if (body.error) throw new Error(body.error.message);
    return body.result;
  };
}

export async function getAccountData(rpc: Rpc, pubkey: string): Promise<Buffer | null> {
  const r = (await rpc("getAccountInfo", [pubkey, { encoding: "base64" }])) as {
    value: { data: [string, string] } | null;
  };
  if (!r?.value) return null;
  return Buffer.from(r.value.data[0], "base64");
}
