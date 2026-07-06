# Reserve V2 Pricing Program Design

## Summary

New Reserve V2 pricing program:

- restrict pricing to only `accepted SPL token mint -> wSOL` flow
- static per-mint fees, similar to `flatslab` pricing program
- a dynamic SOL-out fee that ramps up as liquid SOL falls below a configured target buffer (e.g. 100k wSOL)
- Fee: input mint static fee + dynamic SOL-out fee
- INF Support: INF mint is a normal accepted asset in the mint fee table
- wSOL must not be an entry in the mint fee table as it would clash with dynamic fee
- Reserve V2 LP mint must not be an entry in the mint fee table


### Supported routes

Supported:
- accepted SPL token mint -> wSOL
- INF mint -> wSOL (INF mint should be in the fee table)

Rejected:
- wSOL -> anything
- LST -> LST
- Reserve V2 LP mint -> anything
- anything -> Reserve V2 LP mint
- input mint == output mint


### Static per-mint fees

Each accepted mint has `input_fee_nanos` in the fee table. Reserve exits charge only the input mint's fee as output is always wSOL.
Although LST fees should be equalized for all mints, the fee table is still useful as a LST mint white-list.

- fee rates are in nanos (1e9 = 100%)
- negative fees not supported (would result in quoted output SOL value > input SOL value)
- missing mint fails
- `SetLstFee` must reject wSOL and the Reserve V2 LP mint


### Dynamic fee

Different from the current Reserve where the dynamic fee is based on the percentage of SOL left in the Reserve pool.
Now the dynamic fee is calculated based on the shortfall from the target SOL balance.

Config:
- `target_liquidity_lamports` | `T`: desired liquid SOL buffer, e.g. 100k SOL
- `base_fee_nanos` | `b`: dynamic fee while post-trade liquid SOL >= target
- `max_fee_nanos` | `m`: dynamic fee at zero liquid SOL

Dynamic fee rate based on post-trade balance, with `post_liquid = wSOL_balance_before - wSOL_out` and `shortfall S = max(T - post_liquid, 0)`:
- post_liquid >= T:  `dynamic_fee = b `
- post_liquid <  T:  `dynamic_fee = b + (m - b) * S / T`

Final fee:
total_fee_nanos = input_static_fee_nanos + effective_dynamic_fee_nanos

If the whole trade stays above or below `target_liquidity_lamports`, `effective_dynamic_fee_nanos` is the curve rate at the post-trade point.
If the trade crosses the target, pricing is piecewise. The portion down to `T` pays `b`, and the portion below `T` pays the ramp fee starting from zero shortfall.

Note:
Rounding favors the pool: fee rates round up, user outputs round down, user inputs round up.

### Guards / constraints

- The quote fails if computed output exceeds the liquid wSOL balance
- The quote fails if the total fee rate is >= 100% or negative
- `target_liquidity_lamports > 0`
- `0 <= base_fee_nanos <= max_fee_nanos < 1e9`

### Stake accounts

At the moment, Reserve V2 does not allow direct stake account -> wSOL

### Example

Target 100k SOL, `b` = 10 bps, `m` = 800 bps:

```text
liquid = 500k: exits pay 10 bps until reserves approach 100k
liquid = 60k:  shortfall 40% -> 10 + 790 * 0.4  ~= 326 bps
liquid = 10k:  shortfall 90% -> 10 + 790 * 0.9  ~= 721 bps
```
