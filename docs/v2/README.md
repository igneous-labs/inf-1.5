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
- `last_release_slot: u64`. Slot where yield was last released. See [release_yield](#release_yield)
- `rps: u64`. Proportion of current `withheld_lamports` that is released to the pool per slot, in the [UQ0.63](<https://en.wikipedia.org/wiki/Q_(number_format)>) 63-bit decimal fixed-point format
- `protocol_fee_lamports: u64`. Field that accumulates unclaimed protocol fees in units of lamports (SOL value) that have not yet been claimed by the protocol fee beneficiary
- `rps_auth: Address`. Authority allowed to set `rps` field.

In general, where in the past `total_sol_value` was used, the semantically equivalent value should be `total_sol_value - withheld_lamports - protocol_fee_lamports` instead.

##### Migration Plan

For all instructions that have write access to the `PoolState`, barring exceptions (see [below](#exceptions-non-migrating-instructions)):

- SyncSolValue
- SwapExactIn
- SwapExactOut
- AddLiquidity
- RemoveLiquidity
- SwapExactInV2 (new)
- SwapExactOutV2 (new)

After verifying identity of the `PoolState` account, the handler will check its `version` field and if it's the old version, perform a one-time migration to the new schema by reallocing the account setting the new fields to their initial value.

If necessary, we will transfer SOL to the account to ensure that it has enough for its new rent-exemption requirements before the program upgrade so that a separate payer accout input is not required.

###### Exceptions: non-migrating Instructions

These instructions have write access to `PoolState` but do not perform the migration procedure

These instructions do not run the migration because they are low-frequency non-user-facing instructions

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
- WithdrawProtocolFeesV2 (new)

#### Yield Release Over Time

##### `release_yield`

[All instructions that run `update_yield`](#update_yield) must run `release_yield` prior to the first `update_yield` so that any newly observed yields do not get early-released, with the following exceptions:

- EndRebalance, because the prior StartRebalance would've ran it in the same slot already

Additionally, the following instructions that affect or are affected by yield release events must run it:

- WithdrawProtocolFeesV2 (new)
- SetRps (new)

Immediately after verification, before running anything else, the instruction will run a `release_yield` subroutine which:

- calc `slots_elapsed = sysvar.clock.slot - pool_state.last_release_slot`
- update `pool_state.withheld_lamports *= (1.0-rps)^slots_elapsed` where `rps` is `pool_state.rps` converted to a rate between 0.0 and 1.0
- have `lamports_released` = decrease in withheld_lamports
  - apply protocol fees to `lamports_released` and increment `pool_state.protocol_fee_lamports` by the fee amount
- update `pool_state.last_release_slot = sysvar.clock.slot` if nonzero `lamports_released`
- if `pool_state.withheld_lamports` changed, self-CPI `LogSigned` to log data about how much yield was released

###### Rounding

Due to rounding, poorly timed calls to `release_yield` might result in more yield withheld than expected if parameters result in a single lamport requiring multiple slots to be released.

To mitigate this, we only update `last_release_slot` if `release_yield` results in a nonzero lamport amount being released.

An alternative is to store `withheld_lamports` with greater precision and round when required but we chose not to do this to (hopefully) reduce complexity.

###### Derivation of `pool_state.withheld_lamports` update rule

Basically works similarly to compound interest in lending programs.

let `y = pool_state.withheld_lamports`, `t = slots_elapsed`, `k = rps` in terms of a rate between 0.0 and 1.0.

We want to release `ky` lamports every slot and we're dealing with discrete units of time in terms of slots, which means `y_new = (1.0-k)y_old` after each slot.

This is a geometric sequence with `a = y` and `r = 1.0 - k`

##### `update_yield`

For instructions that involve running at least 1 SyncSolValue procedure:

- SyncSolValue
- SwapExactIn
- SwapExactOut
- AddLiquidity
- RemoveLiquidity
- SetSolValueCalculator
- StartRebalance
- EndRebalance
- SwapExactInV2 (new)
- SwapExactOutV2 (new)

Right before the end of the instruction, it will run a `update_yield` subroutine which:

- Compare `end_total_sol_value` with `start_total_sol_value`
- If theres an increase (yield was observed)
  - Increment `withheld_lamports` by same amount
- If theres a decrease (loss was observed)

  - Decrement by the same amount, saturating, from the following quantities in order
    - `withheld_lamports`
    - `protocol_fee_lamports` if `withheld_lamports` balance is not enough to cover decrement
  - Effect of using any previously accumulated yield and protocol fees to soften the loss
  - Enforces the invariant that the pool is never insolvent for LPers

- In both cases, self-CPI `LogSigned` to log data about how much yield/loss was observed

Special-cases:

- Swaps that add or remove liquidity, changing the INF supply. Instead of comparing `end_total_sol_value` with `start_total_sol_value`, an increment = fee charged by the swap will be added directly.
- EndRebalance. Instead of comparing `end_total_sol_value` with `start_total_sol_value`, `end_total_sol_value` is compared with the `old_total_sol_value` stored in the `RebalanceRecord` instead.

#### Deferred Minting Of Protocol Fees

See [new `WithdrawProtocolFeesV2` instruction below](#withdrawprotocolfeesv2).

The old `WithdrawProtocolFees` instruction will be preserved unchanged to withdraw already accumulated old protocol fees.

#### Unification of Swap & Liquidity instruction

See new [`SwapExactInV2`](#swapexactinv2) and [`SwapExactOutV2`](#swapexactoutv2) instructions below.

To preserve backward compatibility, the current swap and liquidity instructions will not change their account and instruction data inputs but their implementation will simply defer to the new V2 instructions.

This also means the complete deprecation of the `PriceLpTokensToMint` and `PriceLpTokensToRedeem` pricing program interface, which can be done without further action because the account inputs for the current pricing program (flatslab) for all 4 pricing program interface instructions are the exact same.

#### Other Changes

- `SetProtocolFee` instruction will take a single `u32` instead of 2 optional `u16`s for updating `pool_state.protocol_fee_nanos`

### Additions

#### Instructions

##### SwapExactInV2

###### Data

| Name           | Value                                                                                                                                                                                                                    | Type |
| -------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---- |
| discriminant   | 23                                                                                                                                                                                                                       | u8   |
| inp_calc_accs  | number of accounts following out_lst_acc to invoke inp token's SOL value calculator program LstToSol with, excluding the interface prefix accounts. First account should be the calculator program itself. 1 if mint=INF | u8   |
| out_calc_accs  | number of accounts following to invoke out token's SOL value calculator program SolToLst with, excluding the interface prefix accounts. First account should be the calculator program itself. 1 if mint=INF             | u8   |
| inp_index      | index of inp_lst in `lst_state_list`. u32::MAX for INF mint                                                                                                                                                              | u32  |
| out_index      | index of out_lst in `lst_state_list`. u32::MAX for INF mint                                                                                                                                                              | u32  |
| min_amount_out | minimum output amount of out_lst expected                                                                                                                                                                                | u64  |
| amount         | amount of inp tokens to swap                                                                                                                                                                                             | u64  |

###### Accounts

Same as [v1](https://github.com/igneous-labs/S/blob/master/docs/s-controller-program/instructions.md#accounts-1), but with protocol fee accumulator account removed.

| Account           | Description                                                                                                                                                                                                                                   | Read/Write (R/W) | Signer (Y/N) |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- | ------------ |
| signer            | Authority of inp_lst_acc. User making the swap.                                                                                                                                                                                               | R                | Y            |
| inp_mint          | Mint being swapped from                                                                                                                                                                                                                       | R                | N            |
| out_mint          | Mint being swapped to                                                                                                                                                                                                                         | R                | N            |
| inp_acc           | user token account being swapped from                                                                                                                                                                                                         | W                | N            |
| out_acc           | user token account to swap to                                                                                                                                                                                                                 | W                | N            |
| inp_token_program | Input token program                                                                                                                                                                                                                           | R                | N            |
| out_token_program | Output token program                                                                                                                                                                                                                          | R                | N            |
| pool_state        | The pool's state singleton PDA                                                                                                                                                                                                                | W                | N            |
| lst_state_list    | Dynamic list PDA of LstStates for each LST in the pool                                                                                                                                                                                        | W                | N            |
| inp_pool_reserves | Input LST reserves token account of the pool. INF mint if inp=INF                                                                                                                                                                             | W                | N            |
| out_pool_reserves | Output LST reserves token account of the pool. INF mint if out=INF                                                                                                                                                                            | W                | N            |
| inp_calc_accs     | Accounts to invoke inp token's SOL value calculator program LstToSol with, excluding the interface prefix accounts. First account should be the calculator program itself. Multiple Accounts. Single unchecked filler account if inp_mint=INF | ...              | ...          |
| out_calc_accs     | Accounts to invoke out token's SOL value calculator program SolToLst with, excluding the interface prefix accounts. First account should be the calculator program itself. Multiple Accounts. Single unchecked filler account if out_mint=INF | ...              | ...          |
| pricing_accs      | Accounts to invoke pricing program PriceExactIn with. First account should be the pricing program itself. Multiple Accounts.                                                                                                                  | ...              | ...          |

###### Procedure

Same as v1, with following changes:

- Works with Liquidity instructions: inp_mint=INF is RemoveLiquidity, out_mint=INF is AddLiquidity
  - changes to instruction data format to support this documented above
  - since the INF program itself does not implement the [SOL value calculator program interface](https://github.com/igneous-labs/S/tree/master/docs/sol-value-calculator-programs), what would be a CPI for other LSTs would be an inline calculation using mint supply and pool_state data instead to calculate the SOL value of INF tokens
  - SyncSolValue is a no-op

##### SwapExactOutV2

Same as [SwapExactInV2](#swapexactinv2), but

- discriminant = 24
- `max_amount_in` instead of `min_amount_out`
- `amount` is amount of dst tokens to receive
- the core part goes like this instead:
  - out_sol_value = LstToSol(amount).max
  - in_sol_value = PriceExactOut(amount, out_sol_value)
  - amount_in = SolToLst(in_sol_value).max

##### WithdrawProtocolFeesV2

###### Data

| Name         | Value | Type |
| ------------ | ----- | ---- |
| discriminant | 25    | u8   |

###### Accounts

| Account                  | Description                                                    | Read/Write (R/W) | Signer (Y/N) |
| ------------------------ | -------------------------------------------------------------- | ---------------- | ------------ |
| pool_state               | The pool's state singleton PDA                                 | W                | N            |
| protocol_fee_beneficiary | The pool's protocol fee beneficiary                            | R                | Y            |
| withdraw_to              | INF token account to withdraw all accumulated protocol fees to | W                | N            |
| inf_mint                 | INF mint                                                       | W                | N            |
| token_program            | Token program                                                  | R                | N            |

###### Procedure

- mints INF proportionally according to current accumulated `pool_state.protocol_fee_lamports` (should be equivalent to adding liquidity of equivalent SOL value)
- reset `pool_state.protocol_fee_lamports` to 0

###### No-op Cases

The instruction succeeds with no state changes (no INF minted, `protocol_fee_lamports` unchanged) in the following cases:

- No `protocol_fee_lamports` to distribute
- Accumulated `protocol_fee_lamports` is insufficient to mint any INF

##### SetRps

Set `pool_state.rps` to a new value.

###### Data

| Name         | Value             | Type         |
| ------------ | ----------------- | ------------ |
| discriminant | 26                | u8           |
| new_rps      | New RPS to set to | UQ0.63 (u64) |

###### Accounts

| Account    | Description                    | Read/Write (R/W) | Signer (Y/N) |
| ---------- | ------------------------------ | ---------------- | ------------ |
| pool_state | The pool's state singleton PDA | W                | N            |
| rps_auth   | The pool's rps auth            | R                | Y            |

##### SetRpsAuth

Set the pool's RPS authority to a new value.

###### Data

| Name         | Value | Type |
| ------------ | ----- | ---- |
| discriminant | 27    | u8   |

###### Accounts

| Account      | Description                                 | Read/Write (R/W) | Signer (Y/N) |
| ------------ | ------------------------------------------- | ---------------- | ------------ |
| pool_state   | The pool's state singleton PDA              | W                | N            |
| signer       | Either the pool's current rps auth or admin | R                | Y            |
| new_rps_auth | New rps auth to set to                      | R                | N            |

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
