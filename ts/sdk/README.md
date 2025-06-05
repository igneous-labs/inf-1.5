# @sanctumso/inf1

Typescript + WASM SDK for Sanctum Infinity program V1.

## Example Usage

```ts
import {
  createSolanaRpc,
  getBase64Encoder,
  type Address,
  type IInstruction,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import {
  accountsToUpdateForTrade,
  initPks,
  quoteTradeExactIn,
  tradeExactInIx,
  updateForTrade,
  type AccountMap,
  type SplPoolAccounts,
} from "@sanctumso/inf1";

const LAINESOL = "LAinEtNLgpmCP9Rvsf5Hn8W6EhNiKLZQti1xfWMLy6X";
const WSOL = "So11111111111111111111111111111111111111112";
const SPL_POOL_ACCOUNTS: SplPoolAccounts = {
  [LAINESOL]: "2qyEeSAWKfU18AFthrF7JA8z8ZCi1yt76Tqs917vwQTV",
  // ...populate rest of `spl lst mint: stake pool addr` data
  // for every single spl lst mint in the INF pool (all 3 spl program deploys)
};

// If out === INF mint, then below code will work the same,
// but the quote and instruction will be for AddLiquidity instead of SwapExactIn.
//
// If inp === INF mint, then below code will work the same,
// but the quote and instruction will be for RemoveLiquidity instead of SwapExactIn.
const SWAP_MINTS = { inp: LAINESOL, out: WSOL };

async function fetchAccountMap(
  rpc: Rpc<SolanaRpcApi>,
  accounts: string[]
): Promise<AccountMap> {
  return Object.fromEntries(
    await Promise.all(
      accounts.map(async (account) => {
        const { value } = await rpc
          .getAccountInfo(account as Address, {
            encoding: "base64",
          })
          .send();
        const v = value!;
        return [
          account,
          {
            data: new Uint8Array(getBase64Encoder().encode(v.data[0])),
            owner: v.owner,
          },
        ];
      })
    )
  );
}

const rpc = createSolanaRpc("https://api.mainnet-beta.solana.com");

// init
const { poolState: poolStateAddr, lstStateList: lstStateListAddr } = initPks();
const initAccs = await fetchAccountMap(rpc, [poolStateAddr, lstStateListAddr]);
const inf = init(
  {
    poolState: initAccs[poolStateAddr],
    lstStateList: initAccs[lstStateListAddr],
  },
  SPL_POOL_ACCOUNTS
);

// update
const updateAccs = await fetchAccountMap(
  rpc,
  accountsToUpdateForTrade(inf, SWAP_MINTS)
);
updateForTrade(inf, SWAP_MINTS, updateAccs);

// quote
const amt = 1_000_000_000n;
const quote = quoteTradeExactIn(inf, {
  amt,
  mints: SWAP_MINTS,
});

// create transaction instruction

// user-provided pubkeys
const signer = ...;
const inpTokenAcc = ...;
const outTokenAcc = ...;

const ixUncasted = tradeExactInIx(inf, {
  amt,
  limit: quote.out,
  mints: SWAP_MINTS,
  signer,
  tokenAccs: {
    inp: inpTokenAcc,
    out: outTokenAcc,
  },
});
// return type is compatible with kit,
// but needs to be casted explicitly
const ix = ixUncasted as unknown as IInstruction;
```

## Build

### Prerequisites

- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)
- `make` (optional, you can just run the `wasm-pack` commands manually)
