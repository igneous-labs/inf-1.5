# @sanctumso/inf1

Typescript + WASM SDK for Sanctum Infinity program V1.

## Example Usage

```ts
import {
  assertAccountsExist,
  createSolanaRpc,
  fetchEncodedAccounts,
  type Address,
  type IInstruction,
  type Rpc,
  type SolanaRpcApi,
} from "@solana/kit";
import {
  accountsToUpdateForTrade,
  init,
  initPks,
  quoteTradeExactIn,
  tradeExactInIx,
  updateForTrade,
  type AccountMap,
  type SplPoolAccounts,
} from "@sanctumso/inf1";
import initSdk from "@sanctumso/inf1";

// The SDK needs to be initialized once globally before it can be used (idempotent).
// For nodejs environments, use
// `import { initSyncEmbed } from "@sanctumso/inf1"; initSyncEmbed();`
// instead
await initSdk();

const LAINESOL = "LAinEtNLgpmCP9Rvsf5Hn8W6EhNiKLZQti1xfWMLy6X";
const WSOL = "So11111111111111111111111111111111111111112";
const SPL_POOL_ACCOUNTS: SplPoolAccounts = new Map(Object.entries({
  [LAINESOL]: "2qyEeSAWKfU18AFthrF7JA8z8ZCi1yt76Tqs917vwQTV",
  // ...populate rest of `spl lst mint: stake pool addr` data
  // for every spl lst mints in the INF pool (all 3 spl program deploys).
  // To support SPL LSTs that are added later on, the `appendSplLsts` fn
  // can be used to add data
}));

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
  const fetched = await fetchEncodedAccounts(rpc, accounts);
  assertAccountsExist(fetched);
  return new Map(
    fetched.map(({ address, data, programAddress }) => [
      address,
      { data, owner: programAddress },
    ])
  );
}

const rpc = createSolanaRpc("https://api.mainnet-beta.solana.com");

// init
const ipks = initPks();
const initAccs = await fetchAccountMap(rpc, ipks);
const inf = init(
  initAccs,
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
