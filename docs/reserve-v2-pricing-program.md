# Reserve V2 Pricing Program

The main Reserve route is:
`accepted SPL token mint -> wSOL`

where `accepted SPL token mint` is an input mint in the fee table, including INF.

The program also prices Reserve V2 liquidity routes through `PriceExactIn` / `PriceExactOut`:

- AddLiquidity: `wSOL -> Reserve V2 LP mint`
- RemoveLiquidity: `Reserve V2 LP mint -> wSOL`

The fee charged on a route is:
`input mint fee + output mint fee`

The input mint fee is curve-based. It uses the mint's base fee for non-wSOL outputs, and its wSOL-utilization curve when the output mint is wSOL.

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
- The utilization curve applies only when `output_mint == wSOL`.
- When `output_mint != wSOL`, the input fee is the input mint's base fee.
- LST-output routes are not explicitly blocked, they can be disabled by setting the output mint fee to 100% (`1_000_000_000` fee nanos).
- Quotes fail when the relevant route or priced segment has total fee >= 100%.
- There is no fixed target-liquidity parameter. The live pool total SOL value is
  the utilization denominator.
- wSOL utilization is computed from the whole Reserve pool, not per mint. If SOLsLST drains wSOL, fees for other accepted mints increase as well.

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

| Name             | Value                                                 | Type    |
| ---------------- | ----------------------------------------------------- | ------- |
| mint             | accepted SPL token mint                               | Address |
| base_fee_nanos   | input fee at and below `threshold_nanos` (knot 0 fee) | u32     |
| threshold_nanos  | wSOL utilization where the ramp starts (knot 0)       | u32     |
| max_fee_nanos    | input fee at 100% wSOL utilization (knot 1 fee)       | u32     |
| output_fee_nanos | static fee charged when this mint is the output       | u32     |

The two knots `(threshold, base_fee)` and `(100%, max_fee)` define a 2-band
input fee curve: a flat band followed by a linear ramp. Ramp steepness is
derived from the knots and is not stored. The bands share a knot, so the curve
is continuous by construction.

`Init` inserts fee-table entries for `LP_MINT` and `WSOL_MINT` with all fees
zero and a valid default threshold (`threshold = 25%`).
Their entries can be updated later with `SetFeeEntry`.

`entries` is sorted by `mint` for binary search. It acts as both the accepted
mint whitelist and the fee table. It is initialized with `LP_MINT` and
`WSOL_MINT` entries, and grows or shrinks with `realloc()` as other mints are
added and removed.

## Pricing Math

Notation:

```
N = 1_000_000_000
pool_sol_value  = pool_state.total_sol_value before the trade
wsol_balance    = wSOL reserve balance before the trade
wsol_out        = wSOL output

normalized rates used by the formulas:
  base_fee_rate   = base_fee_nanos / N
  max_fee_rate    = max_fee_nanos / N
  output_fee_rate = output_fee_nanos / N

  threshold_utilization = threshold_nanos / N

knots (util, rate):
  k0 = (threshold_utilization, base_fee_rate)
  k1 = (1, max_fee_rate)

the ramp band runs from knot k0 to knot k1
```

Validation:

```
pool_sol_value > 0
0 <= base_fee_nanos <= max_fee_nanos < N
0 <= threshold_nanos < N
0 <= output_fee_nanos <= N
```

The formulas below use real-number arithmetic for clarity, conservative rounding is applied where needed: fee rates and required input SOL value round up, and output SOL value rounds down.

### Utilization Curve

wSOL utilization is based on the pool total SOL value:
`wsol_utilization = (pool_sol_value - wsol_balance) / pool_sol_value`

The per-mint curve is the 2-band piecewise-linear curve through the knots:

```
if wsol_utilization <= threshold_utilization:
  input_fee_rate = base_fee_rate
else:
  input_fee_rate =
    base_fee_rate
    + (max_fee_rate - base_fee_rate)
      * (wsol_utilization - threshold_utilization)
      / (1 - threshold_utilization)
```

The curve is flat at `base_fee_rate` until `threshold_utilization`, then rises
linearly to `max_fee_rate` at 100% utilization.

### Band Pricing For wSOL Output

A wSOL-output trade consumes a range of utilization. The range is cut at the threshold if it crosses from the flat band into the ramp band, and each resulting piece is priced at its own midpoint fee:

```
piece_mid_utilization = (piece_start_used + piece_end_used) / (2 * pool_sol_value)
piece_input_fee = input_fee_rate at piece_mid_utilization
piece_total_fee = piece_input_fee + output_fee_rate
```

For a known wSOL output `wsol_out`:

```
fail if wsol_out > wsol_balance or wsol_balance > pool_sol_value

used_before = pool_sol_value - wsol_balance
used_after  = used_before + wsol_out

piece boundaries in SOL value:
  flat_fee_limit = pool_sol_value * threshold_utilization
```

Split `[used_before, used_after]` at the boundaries it crosses, producing up to
2 pieces (flat band + ramp band), and price each piece separately.

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

| Account       | Description                      | R/W | Signer |
| ------------- | -------------------------------- | --- | ------ |
| pricing_state | `PricingState` PDA               | R   | N      |
| pool_state    | Reserve V2 `PoolStateV2` account | R   | N      |
| wSOL_reserves | Reserve V2 wSOL reserve ATA      | R   | N      |

Shared checks:

- `pricing_state == PRICING_STATE_PDA`
- `pool_state == POOL_STATE`
- `pool_state.total_sol_value > 0`
- `wSOL_reserves == RESERVE_V2_WSOL_RESERVES`
- `input_mint != output_mint`
- input mint and output mint both have `entries`
- if either mint is `LP_MINT` route must be `wSOL -> LP_MINT` or `LP_MINT -> wSOL`

### PriceExactIn

Prices an exact input route.

#### Data

| Name         | Value                         | Type |
| ------------ | ----------------------------- | ---- |
| discriminant | 0                             | u8   |
| amount       | amount of the input token     | u64  |
| sol_value    | SOL value of the input amount | u64  |

#### Return Data

| Name   | Value            | Type |
| ------ | ---------------- | ---- |
| result | output SOL value | u64  |

If `output_mint != wSOL`, the input fee is the input mint base fee:

```
total_fee_nanos = base_fee_nanos + output_fee_nanos
fail if total_fee_nanos >= N
output_sol_value = input_sol_value * (N - total_fee_nanos) / N
```

If `output_mint == wSOL`:

1. Starting from the current utilization, for each piece compute the cost of the full remaining piece with the exact-out formula.
2. If the remaining input covers it, consume it and advance to the next knot. Otherwise solve the final partial piece with the linear closed form below and stop.
3. Check that `PriceExactOut` with output gives a required input <= input_sol_value, else fail.

Flat piece:

```
flat_total_fee = output_fee_rate + base_fee_rate
fail if flat_total_fee >= 1

flat_amount = max(flat_fee_limit - used_before, 0)
cost_to_threshold = flat_amount / (1 - flat_total_fee)

if input_sol_value <= cost_to_threshold:
  output_sol_value = input_sol_value * (1 - flat_total_fee)
else:
  flat_output = flat_amount
  input_left  = input_sol_value - cost_to_threshold
```

Partial ramp piece:

Within the ramp band the fee is linear in the output, so for a candidate output
`x` starting at used position `p`:

```
entry_fee = input_fee_rate at utilization (p / pool_sol_value)

slope_per_sol =
  (max_fee_rate - base_fee_rate)
  / ((1 - threshold_utilization) * pool_sol_value)

midpoint_total_fee(x) = output_fee_rate + entry_fee + slope_per_sol * x / 2
```

Exact-in has the circular form `x = input_left * (1 - midpoint_total_fee(x))`.

```
fail if output_fee_rate + entry_fee >= 1

x = input_left * (1 - output_fee_rate - entry_fee)/ (1 + input_left * slope_per_sol / 2)
```

### PriceExactOut

Prices the input SOL value required for an exact output route.

#### Data

| Name         | Value                             | Type |
| ------------ | --------------------------------- | ---- |
| discriminant | 1                                 | u8   |
| amount       | amount of the output token        | u64  |
| sol_value    | SOL value of the requested output | u64  |

#### Return Data

| Name   | Value                    | Type |
| ------ | ------------------------ | ---- |
| result | required input SOL value | u64  |

If `output_mint != wSOL`, the input fee is the input mint's base fee:

```
total_fee_nanos = base_fee_nanos + output_fee_nanos
fail if total_fee_nanos >= N
input_sol_value = output_sol_value * N / (N - total_fee_nanos)
```

If `output_mint == wSOL`, split `output_sol_value` into flat/ramp pieces if it
crosses the threshold. For each piece:

```
piece_input = piece_output / (1 - piece_total_fee)
```

Return the sum of all piece inputs, each rounded up. Quote fails if
`output_sol_value > wsol_balance` or any piece has `piece_total_fee >= 1`.

### Deprecated LP Compatibility

`PriceLpTokensToMint` and `PriceLpTokensToRedeem` are unsupported.
Reserve V2 liquidity routes use `PriceExactIn` and `PriceExactOut` with `LP_MINT` instead.

### Reserve V2 LP Routes

Reserve V2 LP routes are required to add / remove liquidity to / from the pool.

AddLiquidity: `input_mint = wSOL` and `output_mint = LP_MINT`
RemoveLiquidity: `input_mint = LP_MINT` and `output_mint = wSOL`

Other LP routes are rejected.

Set `LP_MINT` fees accordingly to operational needs.

## Admin Instructions

Only `pricing_state.admin` can execute admin instructions after initialization.
State resizing follows the flatslab slab pattern.

| Instruction      | Disc | Data                          | Notes                                                                              |
| ---------------- | ---- | ----------------------------- | ---------------------------------------------------------------------------------- |
| `Init`           | 255  | none                          | create `PricingState`, set hardcoded initial admin, initialize LP and wSOL entries |
| `SetAdmin`       | 254  | none                          | rotate admin to `new_admin` account                                                |
| `SetFeeEntry`    | 253  | `FeeEntry` fields except mint | insert/update sorted fee-table entry                                               |
| `RemoveFeeEntry` | 252  | none                          | remove non-LP entry if present, success if missing                                 |

### `Init` Accounts

| Account        | Description                 | R/W | Signer |
| -------------- | --------------------------- | --- | ------ |
| initial_admin  | hardcoded initial admin     | R   | Y      |
| payer          | funds `pricing_state`       | W   | Y      |
| pricing_state  | `PricingState` PDA          | W   | N      |
| system_program | system program for creation | R   | N      |

### `SetAdmin` Accounts

| Account       | Description                   | R/W | Signer |
| ------------- | ----------------------------- | --- | ------ |
| admin         | current `pricing_state.admin` | R   | Y      |
| pricing_state | `PricingState` PDA            | W   | N      |
| new_admin     | new admin address to store    | R   | N      |

### `SetFeeEntry` Accounts

| Account        | Description                        | R/W | Signer |
| -------------- | ---------------------------------- | --- | ------ |
| admin          | current `pricing_state.admin`      | R   | Y      |
| payer          | funds account growth if needed     | W   | Y      |
| pricing_state  | `PricingState` PDA                 | W   | N      |
| mint           | accepted SPL token mint to set     | R   | N      |
| system_program | system program for realloc funding | R   | N      |

### `RemoveFeeEntry` Accounts

| Account        | Description                       | R/W | Signer |
| -------------- | --------------------------------- | --- | ------ |
| admin          | current `pricing_state.admin`     | R   | Y      |
| pricing_state  | `PricingState` PDA                | W   | N      |
| mint           | accepted SPL token mint to remove | R   | N      |
| refund_rent_to | receives rent from shrink         | W   | N      |

Admin constraints:

- `pricing_state == PRICING_STATE_PDA`.
- `Init` requires `initial_admin.key == INIT_ADMIN` and `initial_admin` signer,
  then stores `pricing_state.admin = INIT_ADMIN`.
- `base_fee_nanos <= max_fee_nanos < N`.
- `threshold_nanos < N`.
- `output_fee_nanos <= N`.
- `RemoveFeeEntry` rejects `mint == LP_MINT` or `mint == WSOL_MINT`.
