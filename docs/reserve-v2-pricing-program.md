# Reserve V2 Pricing Program

The main Reserve route is:
`accepted SPL token mint -> wSOL`

where `accepted SPL token mint` is an input mint in the fee table, including INF.

The program also prices Reserve V2 liquidity routes through `PriceExactIn` / `PriceExactOut`:

- AddLiquidity: `wSOL -> Reserve V2 LP mint`
- RemoveLiquidity: `Reserve V2 LP mint -> wSOL`

Fees compose by retained SOL value:
`final_retained = input_retained * output_retained`
where `retained = 1 - fee`.

Equivalently:
`effective_fee = input_fee + output_fee - input_fee * output_fee`

The input mint fee is per-mint and may be utilization-based.

Important policy:

- The fee table is the accepted-mint whitelist.
- Normal LSTs can be equalized by using the same per-mint curve config.
- Special mints, such as SOLsLST, can use a different per-mint curve config.
- INF is accepted by adding the INF mint to the fee table.
- `LP_MINT` is a hardcoded program constant. `Init` inserts fee-table entries for
  `LP_MINT` and wSOL, for Reserve V2 liquidity routes to work.
- Direct native stake account -> wSOL is out of scope.

## Core Rules

- Fees are unsigned nanos, where `1_000_000_000` nanos = 100%.
- Negative fees are unsupported.
- Only accepted mints in `entries` can use the main Reserve route.
- Every route composes an input-side fee with the output mint's `output_fee_nanos`.
- If `input_mint != wSOL` and `output_mint` is wSOL or `LP_MINT`, the input-side fee is the input mint's range-based wSOL-utilization curve.
- Otherwise, the input-side fee is the input mint's `base_fee_nanos`.
- LST-output routes are not explicitly blocked, they can be disabled by setting the output mint fee to 100% (`1_000_000_000` fee nanos).
- Quotes fail when the relevant route or priced segment has zero retained value.
- There is no fixed target-liquidity parameter. The live pool total SOL value is
  the utilization denominator.
- wSOL utilization is computed from the whole Reserve pool, not per mint. If SOLsLST drains wSOL, fees for other accepted mints increase as well.
- Use a 100% base fee or output fee to disable a route.

## Accounts

### Program Constants

| Name                       | Value                          |
| -------------------------- | ------------------------------ |
| `INIT_ADMIN`               | hardcoded initial admin        |
| `PRICING_STATE_PDA`        | PDA ["p"]                      |
| `WSOL_MINT`                | wSOL mint                      |
| `LP_MINT`                  | Reserve V2 LP mint             |
| `POOL_STATE`               | Reserve V2 pool state          |
| `RESERVE_V2_WSOL_RESERVES` | ATA(`POOL_STATE`, `WSOL_MINT`) |

### PricingState

The singleton is located at `PRICING_STATE_PDA`.

| Name    | Value                                            | Type       |
| ------- | ------------------------------------------------ | ---------- |
| admin   | authority allowed to update config and fee table | Address    |
| entries | sorted packed slice of `FeeEntry`                | `FeeEntry` |

`FeeEntry`:

| Name                | Value                                           | Type    |
| ------------------- | ----------------------------------------------- | ------- |
| mint                | accepted SPL token mint                         | Address |
| threshold_nanos     | wSOL utilization of the middle knot             | u32     |
| base_fee_nanos      | input fee at 0% wSOL utilization                | u32     |
| threshold_fee_nanos | input fee at `threshold_nanos` wSOL utilization | u32     |
| max_fee_nanos       | input fee at 100% wSOL utilization              | u32     |
| output_fee_nanos    | static fee charged when this mint is the output | u32     |

The three knots `(0%, base_fee)`, `(threshold, threshold_fee)`, and `(100%, max_fee)` define a 2-band input fee curve.

`Init` inserts fee-table entries for `LP_MINT` and `WSOL_MINT` with all fees
zero and a valid default threshold (`threshold = 33.33% / one-third`). Their
entries can be updated later with `SetFeeEntry`.

`LP_MINT` input curve should be high enough that `LP_MINT -> wSOL` cannot be used as a cheaper route for direct `LST -> wSOL`.

`entries` is sorted by `mint` for binary search. It acts as both the accepted mint whitelist and the fee table. It is initialized with `LP_MINT` and
`WSOL_MINT` entries, and grows or shrinks with `realloc()` as other mints are added and removed.

## Pricing Math

Notation:

```
N = 1_000_000_000
pool_sol_value  = pool_state.total_sol_value before the trade
wsol_balance    = wSOL reserve balance before the trade
wsol_out        = wSOL output

normalized rates used by the formulas:
  base_fee_rate      = base_fee_nanos / N
  threshold_fee_rate = threshold_fee_nanos / N
  max_fee_rate       = max_fee_nanos / N
  output_fee_rate    = output_fee_nanos / N

  threshold_utilization = threshold_nanos / N

knots (util, rate):
  k0 = (0, base_fee_rate)
  k1 = (threshold_utilization, threshold_fee_rate)
  k2 = (1, max_fee_rate)

the input fee curve has two bands: k0 -> k1 and k1 -> k2

curve fields are read from the input mint's entry
output_fee is read from the output mint's entry
```

Validation:

```
pool_sol_value > 0
wsol_balance <= pool_sol_value
0 <= base_fee_nanos <= threshold_fee_nanos <= max_fee_nanos <= N
0 < threshold_nanos < N
0 <= output_fee_nanos <= N
```

The formulas below use real-number arithmetic for clarity, conservative rounding is applied where needed: fee rates and required input SOL value round up, and output SOL value rounds down.

For any priced piece:

```
input_retained_rate  = 1 - input_fee_rate
output_retained_rate = 1 - output_fee_rate
piece_retained_rate  = input_retained_rate * output_retained_rate
effective_fee_rate   = 1 - piece_retained_rate
```

This avoids sum-of-fees overflow: 50% input fee and 60% output fee produce
`0.5 * 0.4 = 0.2` retained value, or an 80% effective fee.

### Utilization Curve

wSOL utilization is based on pool total SOL value:
`wsol_utilization = (pool_sol_value - wsol_balance) / pool_sol_value`.

If `wsol_balance > pool_sol_value`, the quote fails as pool accounting state is inconsistent.

The per-mint curve is the 2-band piecewise-linear curve through the knots:

```
if wsol_utilization <= threshold_utilization:
  input_fee_rate = base_fee_rate
    + (threshold_fee_rate - base_fee_rate)
      * wsol_utilization
      / threshold_utilization
else:
  input_fee_rate =
    threshold_fee_rate
    + (max_fee_rate - threshold_fee_rate)
      * (wsol_utilization - threshold_utilization)
      / (1 - threshold_utilization)
```

The first band can be made flat by setting `threshold_fee_rate = base_fee_rate`.

### Band Pricing For wSOL / LP Output

A non-wSOL input trade to wSOL or LP consumes a range of utilization. The range
is cut at the threshold if it crosses from the lower band into the upper band,
and each resulting piece is priced at its own midpoint fee:

```
piece_mid_utilization = (piece_start_used + piece_end_used) / (2 * pool_sol_value)
piece_input_fee = input_fee_rate at piece_mid_utilization
piece_input_retained  = 1 - piece_input_fee
piece_output_retained = 1 - output_fee_rate
piece_retained        = piece_input_retained * piece_output_retained
```

The midpoint method is used instead of integral pricing, so no `exp` / `ln` math is required on-chain.
For direct `LST -> wSOL`, splitting one larger swap into smaller swaps should
not improve the aggregate quote, ignoring rounding.

For a known range-priced output `output_sol_value`:

```
fail if output_sol_value > wsol_balance

used_before = pool_sol_value - wsol_balance
used_after  = pool_sol_value - (wsol_balance - output_sol_value)

piece boundaries in SOL value:
  threshold_limit = pool_sol_value * threshold_utilization
```

Split `[used_before, used_after]` at the boundaries it crosses, producing up to
2 pieces, and price each piece separately.

## Instructions

### Common Interface

| Instruction             | Meaning                                                |
| ----------------------- | ------------------------------------------------------ |
| `PriceExactIn`          | input `sol_value` -> output SOL value                  |
| `PriceExactOut`         | desired output `sol_value` -> required input SOL value |
| `PriceLpTokensToMint`   | unsupported instruction                                |
| `PriceLpTokensToRedeem` | unsupported instruction                                |

Interface-specific accounts are **bolded** in each instruction's account table.

Shared checks:

- `pricing_state == PRICING_STATE_PDA`
- `pool_state == POOL_STATE`
- `pool_state.total_sol_value > 0`
- `wSOL_reserves == RESERVE_V2_WSOL_RESERVES`
- `input_mint != output_mint`
- input mint and output mint both have `entries`

### PriceExactIn

Prices an exact input route.

#### Data

| Name         | Value                         | Type |
| ------------ | ----------------------------- | ---- |
| discriminant | 0                             | u8   |
| amount       | amount of the input token     | u64  |
| sol_value    | SOL value of the input amount | u64  |

#### Accounts

| Account         | Description                      | R/W | Signer |
| --------------- | -------------------------------- | --- | ------ |
| **input_mint**  | mint of the input token          | R   | N      |
| **output_mint** | mint of the output token         | R   | N      |
| pricing_state   | `PricingState` PDA               | R   | N      |
| pool_state      | Reserve V2 `PoolStateV2` account | R   | N      |
| wSOL_reserves   | Reserve V2 wSOL reserve ATA      | R   | N      |

#### Return Data

| Name   | Value            | Type |
| ------ | ---------------- | ---- |
| result | output SOL value | u64  |

#### Procedure

If `input_mint == wSOL` or `output_mint` is neither wSOL nor `LP_MINT`, the
input fee is the input mint base fee:

```
input_retained_nanos  = N - base_fee_nanos
output_retained_nanos = N - output_fee_nanos
fail if input_retained_nanos == 0 or output_retained_nanos == 0

route_retained_nanos = input_retained_nanos * output_retained_nanos / N
output_sol_value = input_sol_value * route_retained_nanos / N
```

If `input_mint != wSOL` and (`output_mint == wSOL || output_mint == LP_MINT`):

1. Starting from the current utilization, split the output range at each knot it crosses.
2. For each piece, compute the cost of the full remaining piece with the exact-out formula.
3. If the remaining input covers it, consume it and advance to the next knot. Otherwise solve the final partial piece with the linear closed form below and stop.
4. Fail if input remains after the final band is fully consumed: the output would exceed the remaining wSOL reserves / equivalent LP cap.

Full piece:

```
piece_mid_utilization = (piece_start_used + piece_end_used) / (2 * pool_sol_value)
piece_input_fee = input_fee_rate at piece_mid_utilization
piece_retained = (1 - piece_input_fee) * (1 - output_fee_rate)
fail if piece_retained == 0

piece_output = piece_end_used - piece_start_used
piece_cost = piece_output / piece_retained
```

Partial piece:

Within a band the fee is linear in the output, so for a candidate output
`x` starting at used position `p`:

```
entry_fee = input_fee_rate at utilization (p / pool_sol_value)

slope_per_sol =
  (band_end_fee - band_start_fee)
  / ((band_end_util - band_start_util) * pool_sol_value)

midpoint_input_retained(x) = 1 - entry_fee - slope_per_sol * x / 2
midpoint_retained(x) = midpoint_input_retained(x) * (1 - output_fee_rate)
```

Exact-in has the circular form `x = input_left * midpoint_retained(x)`.

```
fail if output_fee_rate >= 1 or entry_fee >= 1

x =
  input_left * (1 - output_fee_rate) * (1 - entry_fee)
  / (1 + input_left * (1 - output_fee_rate) * slope_per_sol / 2)
```

### PriceExactOut

Prices the input SOL value required for an exact output route.

#### Data

| Name         | Value                             | Type |
| ------------ | --------------------------------- | ---- |
| discriminant | 1                                 | u8   |
| amount       | amount of the output token        | u64  |
| sol_value    | SOL value of the requested output | u64  |

#### Accounts

| Account         | Description                      | R/W | Signer |
| --------------- | -------------------------------- | --- | ------ |
| **input_mint**  | mint of the input token          | R   | N      |
| **output_mint** | mint of the output token         | R   | N      |
| pricing_state   | `PricingState` PDA               | R   | N      |
| pool_state      | Reserve V2 `PoolStateV2` account | R   | N      |
| wSOL_reserves   | Reserve V2 wSOL reserve ATA      | R   | N      |

#### Return Data

| Name   | Value                    | Type |
| ------ | ------------------------ | ---- |
| result | required input SOL value | u64  |

#### Procedure

If `input_mint == wSOL` or `output_mint` is neither wSOL nor `LP_MINT`, the
input fee is the input mint's base fee:

```
input_retained_nanos  = N - base_fee_nanos
output_retained_nanos = N - output_fee_nanos
fail if input_retained_nanos == 0 or output_retained_nanos == 0

route_retained_nanos = input_retained_nanos * output_retained_nanos / N
input_sol_value = output_sol_value * N / route_retained_nanos
```

If `input_mint != wSOL` and (`output_mint == wSOL || output_mint == LP_MINT`),
split `output_sol_value` into pieces if it crosses a knot. For each piece:

```
piece_input = piece_output / piece_retained
```

Return the sum of all piece inputs, each rounded up. Quote fails if
`output_sol_value > wsol_balance` or any piece has zero retained value.

### RemoveFeeEntry

Removes a non-`LP_MINT`, non-`WSOL_MINT` entry from the fee table

#### Data

| Name         | Value | Type |
| ------------ | ----- | ---- |
| discriminant | 252   | u8   |

#### Accounts

| Account        | Description                       | R/W | Signer |
| -------------- | --------------------------------- | --- | ------ |
| pricing_state  | `PricingState` PDA                | W   | N      |
| admin          | current `pricing_state.admin`     | R   | Y      |
| mint           | accepted SPL token mint to remove | R   | N      |
| refund_rent_to | receives rent from shrink         | W   | N      |

### SetAdmin

Rotates the `pricing_state.admin` to a new address

#### Data

| Name         | Value | Type |
| ------------ | ----- | ---- |
| discriminant | 254   | u8   |

#### Accounts

| Account       | Description                   | R/W | Signer |
| ------------- | ----------------------------- | --- | ------ |
| pricing_state | `PricingState` PDA            | W   | N      |
| admin         | current `pricing_state.admin` | R   | Y      |
| new_admin     | new admin address to store    | R   | N      |

### Init

Permissionless one-time initialization

#### Data

| Name         | Value | Type |
| ------------ | ----- | ---- |
| discriminant | 255   | u8   |

#### Accounts

| Account       | Description           | R/W | Signer |
| ------------- | --------------------- | --- | ------ |
| payer         | funds `pricing_state` | W   | Y      |
| pricing_state | `PricingState` PDA    | W   | N      |

#### Procedure

1. Create `PricingState` PDA account
2. Init account to hardcoded initial admin and fee-table entries for `LP_MINT` and `WSOL_MINT`

### Deprecated LP Compatibility

`PriceLpTokensToMint` and `PriceLpTokensToRedeem` are unsupported.
Reserve V2 liquidity routes use `PriceExactIn` and `PriceExactOut` with `LP_MINT` instead.

### Reserve V2 LP Routes

Reserve V2 LP routes are required to add / remove liquidity to / from the pool.

Examples:

| Route           | Fee behavior                            |
| --------------- | --------------------------------------- |
| LST -> wSOL     | LST input curve + wSOL output fee       |
| LST -> LP_MINT  | LST input curve + `LP_MINT` output fee  |
| wSOL -> LP_MINT | wSOL base fee + `LP_MINT` output fee    |
| LP_MINT -> wSOL | `LP_MINT` input curve + wSOL output fee |
| LP_MINT -> LST  | `LP_MINT` base fee + LST output fee     |
| wSOL -> LST     | wSOL base fee + LST output fee          |
| LST_A -> LST_B  | LST_A base fee + LST_B output fee       |

Intended production config:

| Mint      | Intended config                                                       |
| --------- | --------------------------------------------------------------------- |
| wSOL      | all fees zero                                                         |
| LP_MINT   | output fee zero, input curve high enough to protect `LP_MINT -> wSOL` |
| LST / INF | output fee 100% to block LST-output routes, input curve set by ops    |

The main concern is that `LST -> LP_MINT -> wSOL` becomes cheaper than the direct `LST -> wSOL` route.

Pricing `LST -> LP_MINT` like the equivalent `LST -> wSOL` route prevents the single-shot bypass.
But splitting into smaller `LST -> LP_MINT` swaps can get cheaper as pool value grows between calls.

The load-bearing mitigation is the `LP_MINT` input curve.
Configure `LP_MINT` input fees high enough such that `LP_MINT -> wSOL` is not cheaper than the direct route even with chunking discount.

## Admin Instructions

Only `pricing_state.admin` can execute admin instructions after initialization.
State resizing follows the flatslab slab pattern.

| Instruction   | Disc | Data                          | Notes                                |
| ------------- | ---- | ----------------------------- | ------------------------------------ |
| `SetFeeEntry` | 253  | `FeeEntry` fields except mint | insert/update sorted fee-table entry |

### `SetFeeEntry` Accounts

| Account        | Description                        | R/W | Signer |
| -------------- | ---------------------------------- | --- | ------ |
| pricing_state  | `PricingState` PDA                 | W   | N      |
| admin          | current `pricing_state.admin`      | R   | Y      |
| mint           | accepted SPL token mint to set     | R   | N      |
| payer          | funds account growth if needed     | W   | Y      |
| system_program | system program for realloc funding | R   | N      |

Admin constraints:

- `pricing_state == PRICING_STATE_PDA`.
- `base_fee_nanos <= threshold_fee_nanos <= max_fee_nanos <= N`.
- `0 < threshold_nanos < N`.
- `output_fee_nanos <= N`.
- `RemoveFeeEntry` rejects `mint == LP_MINT` or `mint == WSOL_MINT`.
