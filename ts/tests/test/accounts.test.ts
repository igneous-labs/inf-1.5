import {
  getPoolState,
  getLstStateList,
  init,
  initPks,
  initSyncEmbed,
  Inf,
  serPoolState,
  serLstStateList,
} from "@sanctumso/inf1";
import { beforeAll, describe, expect, it } from "vitest";
import {
  fetchAccountMap,
  localRpc,
  LST_STATE_LIST_ID,
  POOL_STATE_ID,
  SPL_POOL_ACCOUNTS,
} from "../utils";
import { type Address, type Rpc, type SolanaRpcApi } from "@solana/kit";

async function splInf(rpc: Rpc<SolanaRpcApi>): Promise<Inf> {
  const pks = initPks();
  const initAccs = await fetchAccountMap(rpc, pks as Address[]);
  // init with SPL_POOL_ACCOUNTS
  return init(initAccs, SPL_POOL_ACCOUNTS);
}

describe("accounts test", () => {
  beforeAll(() => initSyncEmbed());

  const rpc = localRpc();

  it("happy path getPoolState", async () => {
    const inf = await splInf(rpc);
    const pool = getPoolState(inf);

    expect(pool).toMatchInlineSnapshot(`
      {
        "admin": "8VE2uJkoheDbJd9rCyKzfXmiMqAS4o1B3XGshEh86BGk",
        "isDisabled": 0,
        "isRebalancing": 0,
        "lpProtocolFeeBps": 1000,
        "lpTokenMint": "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
        "pricingProgram": "s1b6NRXj6ygNu1QMKXh2H9LUR2aPApAAm1UQ2DjdhNV",
        "protocolFeeBeneficiary": "EeQmNqm1RcQnee8LTyx6ccVG9FnR8TezQuw2JXq2LC1T",
        "rebalanceAuthority": "GFHMc9BegxJXLdHJrABxNVoPRdnmVxXiNeoUCEpgXVHw",
        "totalSolValue": 741676030733161n,
        "tradingProtocolFeeBps": 1000,
        "version": 1,
      }
    `);
  });

  it("round trip serPoolState", async () => {
    const data = serPoolState(await splInf(rpc));
    // create a new inf, but overriding fetched account data
    const initAccs = await fetchAccountMap(rpc, initPks() as Address[]);
    initAccs.set(POOL_STATE_ID, {
      ...initAccs.get(POOL_STATE_ID)!,
      data,
    });
    const rt = serPoolState(init(initAccs, SPL_POOL_ACCOUNTS));
    expect(data).toStrictEqual(rt);
  });

  it("happy path getLstStateList", async () => {
    const inf = await splInf(rpc);
    const lstStates = getLstStateList(inf);
    expect(lstStates.length).toBeGreaterThan(0);

    expect(lstStates).toMatchInlineSnapshot(`
      [
        {
          "isInputDisabled": 0,
          "mint": "So11111111111111111111111111111111111111112",
          "poolReservesBump": 255,
          "protocolFeeAccumulatorBump": 255,
          "solValue": 13414450670097n,
          "solValueCalculator": "wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE",
        },
        {
          "isInputDisabled": 0,
          "mint": "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
          "poolReservesBump": 254,
          "protocolFeeAccumulatorBump": 253,
          "solValue": 30344n,
          "solValueCalculator": "1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR",
        },
        {
          "isInputDisabled": 0,
          "mint": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
          "poolReservesBump": 254,
          "protocolFeeAccumulatorBump": 255,
          "solValue": 14651n,
          "solValueCalculator": "mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP",
        },
        {
          "isInputDisabled": 0,
          "mint": "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
          "poolReservesBump": 253,
          "protocolFeeAccumulatorBump": 249,
          "solValue": 98025942575128n,
          "solValueCalculator": "ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo",
        },
      ]
    `);
  });

  it("round trip serLstStateList", async () => {
    const data = serLstStateList(await splInf(rpc));
    // create a new inf, but overriding fetched account data
    const initAccs = await fetchAccountMap(rpc, initPks() as Address[]);
    initAccs.set(LST_STATE_LIST_ID, {
      ...initAccs.get(LST_STATE_LIST_ID)!,
      data,
    });
    const rt = serLstStateList(init(initAccs, SPL_POOL_ACCOUNTS));
    expect(data).toStrictEqual(rt);
  });
});
