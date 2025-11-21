# v2

Documentation of v2 changes and migration procedure

## Controller Program

### Changes

The main changes are

1. Introduction of system to release all yields over time
2. Deferred minting of protocol fees
3. Unification of Swap & Liquidity instructions into a single Swap instruction

#### `PoolState` schema

Change:

- merge `trading_protocol_fee_bps` and `lp_protocol_fee_bps` into a single `protocol_fee_nanos: u32` field, where nanos=(1 / 10^9)

Add fields:

- `withheld_lamports: u64`. Field that records accrued yield in units of lamports (SOL value) that have not yet been released to the pool
- `last_release_slot: u64`. Slot where yield was last released, which happens on all instructions that have writable access to the pool
- `rps_picos: u64`. Proportion of current `withheld_lamports` that is released to the pool per slot, in terms of picos (1 / 10^12)
- `protocol_fee_lamports: u64`. Field that accumulates unclaimed protocol fees in units of lamports (SOL value) that have not yet been claimed by the protocol fee beneficiary

In general, where in the past `total_sol_value` was used, the semantically equivalent value should be `total_sol_value - withheld_lamports - protocol_fee_lamports` instead.

#### Yield Release Over Time

##### `release_yield`

For all instructions that have write access to the `PoolState`:

- SyncSolValue
- SwapExactIn
- SwapExactOut
- AddLiquidity
- RemoveLiquidity
- SetSolValueCalculator
- SetAdmin
- SetProtocolFee
- SetProtocolFeeBeneficiary
- SetPricingProgram
- DisablePool
- EnablePool
- StartRebalance
- EndRebalance
- SetRebalanceAuthority

Immediately after verification, before running anything else, the instruction will run a `release_yield` subroutine which:

- calc `slots_elapsed = sysvar.clock.slot - pool_state.last_release_slot`
- update `pool_state.withheld_lamports *= (1.0-rps_picos)^slots_elapsed` where `rps_picos` is `pool_state.rps_picos` converted to a rate between 0.0 and 1.0
- update `pool_state.last_release_slot = sysvar.clock.slot`
- if `pool_state.withheld_lamports` changed, self-CPI `LogSigned` to log data about how much yield was released

##### `update_yield`

For instructions that involve running at least 1 SyncSolValue procedure, apart from `AddLiquidity` and `RemoveLiquidity`:

- SyncSolValue
- SwapExactIn
- SwapExactOut
- SetSolValueCalculator
- StartRebalance
- EndRebalance

Right before the end of the instruction, it will run a `update_yield` subroutine which:

- Compare `pool.total_sol_value` at the start of the instruction with that at the end of the instruction
- If theres an increase (yield was observed)
  - Divide the increase according to `pool_state.protocol_fee_nanos`
  - Increment `pool_state.protocol_fee_lamports` by protocol fee share
  - Increment `pool_state.withheld_lamports` by non-protocol fee share
- If theres a decrease (loss was observed)
  - decrement `pool_state.withheld_lamports` by the equivalent value (saturating). This has the effect of using any previously accumulated yield to soften the loss
- In both cases, self-CPI `LogSigned` to log data about how much yield/loss was observed.

`AddLiquidity` and `RemoveLiquidity` instructions require special-case handling because they modify both `pool.total_sol_value` and INF mint supply, so yields and losses need to be counted using the differences between the ratio of the 2 before and after.

- Calc `normalized_start_sol_value = start_total_sol_value * end_inf_supply / start_inf_supply` (ceil div)
- Run same procedure as above but compare `end_total_sol_value` against `normalized_start_sol_value`

##### Appendix: derivation of `pool_state.withheld_lamports` update rule

Basically works similarly to compound interest in lending programs.

let `y = pool_state.withheld_lamports`, `t = slots_elapsed`, `k = rps_picos` in terms of a rate between 0.0 and 1.0.

We want to release `ky` lamports every slot and we're dealing with discrete units of time in terms of slots, which means `y_new = (1.0-k)y_old` after each slot.

This is a geometric sequence with `a = y` and `r = 1.0 - k`

#### Deferred Minting Of Protocol Fees

See new `WithdrawProtocolFeesV2` instruction below.

#### Unification of Swap & Liquidity instruction

See new `SwapExactInV2` and `SwapExactOutV2` instructions below.

### Additions

#### Instructions

##### LogSigned

No-op instruction for self-CPI for logging/indexing purposes

###### Data

| Name         | Value | Type |
| ------------ | ----- | ---- |
| discriminant | 255   | u8   |

###### Accounts

| Account    | Description                    | Read/Write (R/W) | Signer (Y/N) |
| ---------- | ------------------------------ | ---------------- | ------------ |
| pool_state | The pool's state singleton PDA | R                | Y            |
