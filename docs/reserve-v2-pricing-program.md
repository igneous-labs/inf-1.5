# Reserve V2 Pricing Program

The main Reserve route is:
`accepted SPL token mint -> wSOL`

where `accepted SPL token mint` is an input mint in the fee table, including INF.

The program also prices Reserve V2 liquidity routes through `PriceExactIn` / `PriceExactOut`:

- AddLiquidity: `wSOL -> Reserve V2 LP mint`
- RemoveLiquidity: `Reserve V2 LP mint -> wSOL`

The fee charged on the main Reserve route is:
`input mint static fee + dynamic reserve-utilization fee`

Important policy:

- The fee table is the accepted-mint whitelist.
- LST static fees should be equalized at initialization, but still stored per mint for whitelisting and future updates.
- INF is accepted by adding the INF mint to the fee table.
- `LP_MINT` is a hardcoded program constant. `Init` inserts fee-table entries for `LP_MINT` and wSOL, for Reserve V2 liquidity routes to work.
- Direct native stake account -> wSOL is out of scope.

## Core Rules

- Fees are unsigned nanos, where `1_000_000_000` nanos = 100%.
- Negative fees are unsupported.
- Only accepted mints in `entries` can use the main Reserve route.
- Dynamic reserve-utilization fees apply only when `output_mint == wSOL`, AddLiquidity uses only the wSOL input static fee.
- `wSOL -> LST` and `LST -> LST` routes are rejected.

## Accounts

### Program Constants

| Name                       | Value                    |
| -------------------------- | ------------------------ |
| `INIT_ADMIN`               | hardcoded initial admin  |
| `PRICING_STATE_PDA`        | PDA ["p"]                |
| `WSOL_MINT`                | wSOL mint                |
| `LP_MINT`                  | Reserve V2 LP mint       |
| `POOL`                     | Reserve V2 pool          |
| `RESERVE_V2_WSOL_RESERVES` | ATA(`POOL`, `WSOL_MINT`) |

### PricingState

The singleton is located at `PRICING_STATE_PDA`.

| Name                      | Value                                            | Type              |
| ------------------------- | ------------------------------------------------ | ----------------- |
| admin                     | authority allowed to update config and fee table | Address           |
| target_liquidity_lamports | desired post-trade wSOL buffer                   | u64               |
| base_fee_nanos            | dynamic fee at/above target liquidity            | u32               |
| max_fee_nanos             | dynamic fee at zero post-trade wSOL liquidity    | u32               |
| entries                   | sorted packed slice of `(mint, input_fee_nanos)` | &[(Address, u32)] |

`Init` inserts fee-table entries for `LP_MINT` and `WSOL_MINT` with `input_fee_nanos = 0`, their fees can be updated later with `SetLstFee`.

`entries` is sorted by `mint` for binary search. It acts as both the accepted mint whitelist and the static input-fee table.
It is initialized with `LP_MINT` and `WSOL_MINT` entries, and grows or shrinks with `realloc()` as other mints are added and removed.

## Pricing Math

Notation:

```
N = 1_000_000_000
T = target_liquidity_lamports
b = base_fee_nanos
m = max_fee_nanos
s = input_static_fee_nanos
L = wSOL reserve balance before the trade
O = wSOL output

a = s + b                         // total fee while reserves stay at/above target
d = m - b                         // additional dynamic fee across the ramp
E = max(L - T, 0)                 // wSOL that can exit before crossing target
F = max(T - L, 0)                 // starting shortfall if already below target
```

Validation:

```
T > 0
0 <= s < N
0 <= b <= m < N
s + m < N
```

### Dynamic Fee Curve

The formulas below use real-number arithmetic for clarity, conservative rounding is applied where needed: fee rates and required input SOL value round up, and output SOL value rounds down.

For a known output `O`, dynamic fee is based on post-trade wSOL liquidity:

```
fail if O > L

post_liquid = L - O
shortfall   = max(T - post_liquid, 0)
dynamic_fee = b + d * shortfall / T
total_fee   = s + dynamic_fee

fail if total_fee >= N
```

If a trade crosses below `T`, pricing is piecewise:

- the portion down to `T` pays `b`
- the portion below `T` pays the ramp fee starting from zero shortfall

## Instructions

### Common Interface

| Instruction             | Disc | Prefix accounts                 | Meaning                                                |
| ----------------------- | ---- | ------------------------------- | ------------------------------------------------------ |
| `PriceExactIn`          | 0    | **input_mint**, **output_mint** | input `sol_value` -> output SOL value                  |
| `PriceExactOut`         | 1    | **input_mint**, **output_mint** | desired output `sol_value` -> required input SOL value |
| `PriceLpTokensToMint`   | 2    | **input_mint**                  | unsupported instruction                                |
| `PriceLpTokensToRedeem` | 3    | **output_mint**                 | unsupported instruction                                |

Interface prefix accounts are **bolded**. Exact pricing instructions append the
same suffix accounts:

| Account       | Description                 | R/W | Signer |
| ------------- | --------------------------- | --- | ------ |
| pricing_state | `PricingState` PDA          | R   | N      |
| wSOL_reserves | Reserve V2 wSOL reserve ATA | R   | N      |

Shared checks:

- `pricing_state == PRICING_STATE_PDA`
- `wSOL_reserves == RESERVE_V2_WSOL_RESERVES`
- `target_liquidity_lamports > 0`
- `base_fee_nanos <= max_fee_nanos < N`
- `input_mint != output_mint`
- route is one of:
  - accepted non-LP mint -> wSOL
  - wSOL -> `LP_MINT`
  - `LP_MINT` -> wSOL

### PriceExactIn

Prices an exact input route.

Data:

- `amt`: input token amount, kept for interface compatibility.
- `sol_value`: input SOL value `I`.

Return:

- output SOL value `O`.

If `output_mint != wSOL`, this is the AddLiquidity route and uses only the input static fee:

```
O = I * (N - s) / N
```

1. Compute the above-target candidate:

```
O_flat = I * (N - a) / N
```

If `O_flat <= E`, return `O_flat`.
No inversion is needed here as fee rate is fixed at `a`.

2. For an entirely below-target input chunk:

Initial fee formula is:

```
total_fee = a + d * (F + O) / T
          = (a * T) / T + d * (F + O) / T
          = (a * T + d * (F + O)) / T

O = I * (N - total_fee) / N
```

Because exact-in output `O` depends on `total_fee`, this solves to:

```
total_fee = N * (a * T + d * (F + I)) / (N * T + d * I)
O         = I * (N - total_fee) / N
```

3. For crossing-target trades:

```
I1 = E * N / (N - a)
O1 = E
I2 = I - I1
O2 = below_target_exact_in(I2, F = 0)
O  = O1 + O2
```

The split is only used when `L > T` and `O_flat > E`.
For wSOL-output routes, quote fails if `total_fee >= N` or `O > L`.

### PriceExactOut

Prices the input SOL value required for an exact output route.

Data:

- `amt`: output token amount, kept for interface compatibility.
- `sol_value`: desired output SOL value `O`.

Return:

- required input SOL value `I`.

If `output_mint != wSOL`, this is the AddLiquidity route and uses only the input static fee:

```
I = O * N / (N - s)
```

If `output_mint == wSOL` and `O > L`, the quote fails.

1. If `O <= E`, the whole trade stays at/above target:

```
I = O * N / (N - a)
```

This is the inverse of the above-target exact-in formula.

2. For an entirely below-target output chunk, use the known-output fee curve:

```
shortfall   = F + O
dynamic_fee = b + d * shortfall / T
total_fee   = s + dynamic_fee
I           = O * N / (N - total_fee)
```

3. For crossing-target trades:

```
O1 = E
O2 = O - O1
I1 = O1 * N / (N - a)
I2 = exact_out_below_target(O2, F = 0)
I  = I1 + I2
```

The split is only used when `L > T` and `O > E`. Quote fails if `total_fee >= N`.

### Deprecated LP Compatibility

`PriceLpTokensToMint` and `PriceLpTokensToRedeem` are unsupported. Reserve V2 liquidity routes use `PriceExactIn` and `PriceExactOut` with `LP_MINT` instead.

### Reserve V2 LP Routes

Reserve V2 LP routes are required to add / remove liquidity to / from the pool.

AddLiquidity: `input_mint = wSOL` and `out_mint = LP_MINT`
RemoveLiquidity: `inp_mint = LP_MINT` and `output_mint = wSOL`

Other LP routes are rejected.

### Admin Instructions

Only `pricing_state.admin` can execute admin instructions after initialization.
State resizing follows the flatslab slab pattern.
The account tables below are intended to guide implementation and common account struct extraction.

| Instruction           | Disc | Data                                                           | Notes                                                                              |
| --------------------- | ---- | -------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| `Init`                | 255  | `target_liquidity_lamports`, `base_fee_nanos`, `max_fee_nanos` | create `PricingState`, set hardcoded initial admin, initialize LP and wSOL entries |
| `SetAdmin`            | 254  | none                                                           | rotate admin to `new_admin` account                                                |
| `SetDynamicFeeParams` | 253  | `target_liquidity_lamports`, `base_fee_nanos`, `max_fee_nanos` | update dynamic fee curve                                                           |
| `SetLstFee`           | 252  | `input_fee_nanos`                                              | insert/update sorted fee-table entry                                               |
| `RemoveLst`           | 251  | none                                                           | remove non-LP entry if present, success if missing                                 |

#### `Init` Accounts

| Account        | Description                 | R/W | Signer |
| -------------- | --------------------------- | --- | ------ |
| initial_admin  | hardcoded initial admin     | R   | Y      |
| payer          | funds `pricing_state`       | W   | Y      |
| pricing_state  | `PricingState` PDA          | W   | N      |
| system_program | system program for creation | R   | N      |

#### `SetAdmin` Accounts

| Account       | Description                   | R/W | Signer |
| ------------- | ----------------------------- | --- | ------ |
| admin         | current `pricing_state.admin` | R   | Y      |
| pricing_state | `PricingState` PDA            | W   | N      |
| new_admin     | new admin address to store    | R   | N      |

#### `SetDynamicFeeParams` Accounts

| Account       | Description                   | R/W | Signer |
| ------------- | ----------------------------- | --- | ------ |
| admin         | current `pricing_state.admin` | R   | Y      |
| pricing_state | `PricingState` PDA            | W   | N      |

#### `SetLstFee` Accounts

| Account        | Description                        | R/W | Signer |
| -------------- | ---------------------------------- | --- | ------ |
| admin          | current `pricing_state.admin`      | R   | Y      |
| payer          | funds account growth if needed     | W   | Y      |
| pricing_state  | `PricingState` PDA                 | W   | N      |
| mint           | accepted SPL token mint to set     | R   | N      |
| system_program | system program for realloc funding | R   | N      |

#### `RemoveLst` Accounts

| Account        | Description                       | R/W | Signer |
| -------------- | --------------------------------- | --- | ------ |
| admin          | current `pricing_state.admin`     | R   | Y      |
| pricing_state  | `PricingState` PDA                | W   | N      |
| mint           | accepted SPL token mint to remove | R   | N      |
| refund_rent_to | receives rent from shrink         | W   | N      |

Admin constraints:

- `pricing_state == PRICING_STATE_PDA`.
- `Init` requires `initial_admin.key == INIT_ADMIN` and `initial_admin` signer, then stores `pricing_state.admin = INIT_ADMIN`.
- `target_liquidity_lamports > 0`.
- `base_fee_nanos <= max_fee_nanos < N`.
- `SetLstFee` requires `input_fee_nanos + current max_fee_nanos < N`.
- `SetDynamicFeeParams` requires every entry's `input_fee_nanos + new max_fee_nanos < N`.
- `RemoveLst` rejects `mint == LP_MINT` or `mint == WSOL_MINT`.
