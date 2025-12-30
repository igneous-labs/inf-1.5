import {
  getPoolState,
  getLstStateList,
  init,
  initPks,
  initSyncEmbed,
  Inf,
  serPoolState,
  serLstStateList,
  setPoolState,
  deserPoolState,
  setLstStateList,
  deserLstStateList,
} from "@sanctumso/inf1";
import { beforeAll, describe, expect, it } from "vitest";
import { fetchAccountMap, localRpc, SPL_POOL_ACCOUNTS } from "../utils";
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

  it("round trip setPoolState getPoolState", async () => {
    const inf = await splInf(rpc);
    const pool = {
      admin: "8VE2uJkoheDbJd9rCyKzfXmiMqAS4o1B3XGshEh86BGk",
      isDisabled: 1,
      isRebalancing: 1,
      lpProtocolFeeBps: 100,
      lpTokenMint: "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
      pricingProgram: "s1b6NRXj6ygNu1QMKXh2H9LUR2aPApAAm1UQ2DjdhNV",
      protocolFeeBeneficiary: "EeQmNqm1RcQnee8LTyx6ccVG9FnR8TezQuw2JXq2LC1T",
      rebalanceAuthority: "GFHMc9BegxJXLdHJrABxNVoPRdnmVxXiNeoUCEpgXVHw",
      totalSolValue: 74167603073316n,
      tradingProtocolFeeBps: 100,
      version: 1,
    };

    setPoolState(inf, pool);

    const newPool = getPoolState(inf);

    expect(pool).toStrictEqual(newPool);
  });

  it("round trip setPoolState serPoolState deserPoolState getPoolState", async () => {
    const inf = await splInf(rpc);
    const pool = {
      admin: "8VE2uJkoheDbJd9rCyKzfXmiMqAS4o1B3XGshEh86BGk",
      isDisabled: 1,
      isRebalancing: 1,
      lpProtocolFeeBps: 100,
      lpTokenMint: "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
      pricingProgram: "s1b6NRXj6ygNu1QMKXh2H9LUR2aPApAAm1UQ2DjdhNV",
      protocolFeeBeneficiary: "EeQmNqm1RcQnee8LTyx6ccVG9FnR8TezQuw2JXq2LC1T",
      rebalanceAuthority: "GFHMc9BegxJXLdHJrABxNVoPRdnmVxXiNeoUCEpgXVHw",
      totalSolValue: 74167603073316n,
      tradingProtocolFeeBps: 100,
      version: 1,
    };

    setPoolState(inf, pool);

    const data = serPoolState(inf);

    const newInf = await splInf(rpc);

    deserPoolState(newInf, data);

    const newPool = getPoolState(newInf);

    expect(pool).toStrictEqual(newPool);
  });

  it("happy path deserPoolState", async () => {
    const inf = await splInf(rpc);
    const pool = {
      admin: "8VE2uJkoheDbJd9rCyKzfXmiMqAS4o1B3XGshEh86BGk",
      isDisabled: 1,
      isRebalancing: 1,
      lpProtocolFeeBps: 100,
      lpTokenMint: "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
      pricingProgram: "s1b6NRXj6ygNu1QMKXh2H9LUR2aPApAAm1UQ2DjdhNV",
      protocolFeeBeneficiary: "EeQmNqm1RcQnee8LTyx6ccVG9FnR8TezQuw2JXq2LC1T",
      rebalanceAuthority: "GFHMc9BegxJXLdHJrABxNVoPRdnmVxXiNeoUCEpgXVHw",
      totalSolValue: 74167603073316n,
      tradingProtocolFeeBps: 100,
      version: 1,
    };

    setPoolState(inf, pool);

    const poolData = serPoolState(inf);

    const newInf = await splInf(rpc);

    deserPoolState(newInf, poolData);

    const newPool = getPoolState(newInf);

    expect(pool).toStrictEqual(newPool);
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

  it("happy path setLstStateList", async () => {
    const inf = await splInf(rpc);

    const lstStates = [
      {
        isInputDisabled: 1,
        mint: "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        poolReservesBump: 255,
        protocolFeeAccumulatorBump: 252,
        solValue: 303444n,
        solValueCalculator: "1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR",
      },
      {
        isInputDisabled: 1,
        mint: "So11111111111111111111111111111111111111112",
        poolReservesBump: 250,
        protocolFeeAccumulatorBump: 251,
        solValue: 1341445067009n,
        solValueCalculator: "wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE",
      },
      {
        isInputDisabled: 1,
        mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        poolReservesBump: 255,
        protocolFeeAccumulatorBump: 255,
        solValue: 146510n,
        solValueCalculator: "mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP",
      },
      {
        isInputDisabled: 1,
        mint: "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        poolReservesBump: 255,
        protocolFeeAccumulatorBump: 240,
        solValue: 9802594257518n,
        solValueCalculator: "ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo",
      },
    ];

    setLstStateList(inf, lstStates);

    let newLstStates = getLstStateList(inf);

    expect(lstStates).toStrictEqual(newLstStates);
  });

  it("happy path serLstStateList", async () => {
    const inf = await splInf(rpc);

    const lstStates = [
      {
        isInputDisabled: 1,
        mint: "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        poolReservesBump: 255,
        protocolFeeAccumulatorBump: 252,
        solValue: 303444n,
        solValueCalculator: "1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR",
      },
    ];

    setLstStateList(inf, lstStates);

    const data = serLstStateList(inf);

    const newInf = await splInf(rpc);

    deserLstStateList(newInf, data);

    const newLstStates = getLstStateList(newInf);

    expect(lstStates).toStrictEqual(newLstStates);
  });

  it("round trip setLstStateList serLstStateList deserLstStateList getLstStateList", async () => {
    const inf = await splInf(rpc);

    const lstStates = [
      {
        isInputDisabled: 1,
        mint: "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj",
        poolReservesBump: 255,
        protocolFeeAccumulatorBump: 252,
        solValue: 303444n,
        solValueCalculator: "1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR",
      },
      {
        isInputDisabled: 1,
        mint: "So11111111111111111111111111111111111111112",
        poolReservesBump: 250,
        protocolFeeAccumulatorBump: 251,
        solValue: 1341445067009n,
        solValueCalculator: "wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE",
      },
      {
        isInputDisabled: 1,
        mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        poolReservesBump: 255,
        protocolFeeAccumulatorBump: 255,
        solValue: 146510n,
        solValueCalculator: "mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP",
      },
      {
        isInputDisabled: 1,
        mint: "jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v",
        poolReservesBump: 255,
        protocolFeeAccumulatorBump: 240,
        solValue: 9802594257518n,
        solValueCalculator: "ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo",
      },
    ];

    setLstStateList(inf, lstStates);

    const lstStateData = serLstStateList(inf);

    const newInf = await splInf(rpc);

    deserLstStateList(newInf, lstStateData);

    const newLstStates = getLstStateList(newInf);

    expect(lstStates).toStrictEqual(newLstStates);
  });
});
