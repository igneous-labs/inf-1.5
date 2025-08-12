# `FlatSlab` Pricing Program

This is basically the same as the [flatfee pricing program](../flatfee/), but with the following changes:

- Instead of `i16` bps, we use a `i32` with `1_000_000_000` as the denominator to calculate rate (instead of `10_000` for bps previously) for more granular control. These 1 / 1_000_000_000 units will be referred to as `nanos`s for the rest of this doc
- All input and output fees are stored in the same static PDA in an array of `(mint, input_fee, output_fee)` sorted by mint i.e. a giant slab. Binary searches are performed to read the fees to price trades for each mint.
- Instead of special-casing `PriceLpTokensToMint` and `PriceLpTokensToRedeem`, the LP token (INF) is simply treated as another mint on the slab. Identity of this LP token mint is hardcoded into the program.
- This slab account also contains a header of a `manager` pubkey that specifies who is authorized to
  - set new `manager`
  - set fees for each mint
  - add and remove mints from the slab

## Accounts

### Slab

The singleton is located at PDA ["slab"].

#### Schema

| Name    | Value                                                                                                                                     | Type                  |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------- | --------------------- |
| manager | The manager authorized to set new manager, update fees, and add/remove mints from the slab                                                | Pubkey                |
| entries | Packed slice of `(mint, input_fee_nanos, output_fee_nanos)`. This slice grows and shrinks with `realloc()` as mints are added and removed | &[(Pubkey, u32, u32)] |

## Instructions

### Common Interface

- Instruction data and return data formats are defined by the interface and cannot be modified
- Interface-specific accounts are **bolded**

#### PriceExactIn

Given an input LST amount and its SOL value, calculate the output SOL value by:

- binary search slab to obtain entries for input and output LST
- calculate total fee in nanos by adding `inp.input_fee_nanos` and `out.output_fee_nanos`
- calculate output LST's sol value after imposing fee by using the calculated fee and the given `sol_value` of input lst

##### Data

| Name         | Value                         | Type |
| ------------ | ----------------------------- | ---- |
| discriminant | 0                             | u8   |
| amount       | amount of the input LST       | u64  |
| sol_value    | SOL value of amount input LST | u64  |

##### Accounts

| Account             | Description            | Read/Write (R/W) | Signer (Y/N) |
| ------------------- | ---------------------- | ---------------- | ------------ |
| **input_lst_mint**  | Mint of the input LST  | R                | N            |
| **output_lst_mint** | Mint of the output LST | R                | N            |
| slab                | slab PDA               | R                | N            |

##### Return Data

| Name   | Value                           | Type |
| ------ | ------------------------------- | ---- |
| result | The calculated output SOL value | u64  |

##### Procedure

#### PriceExactOut

Given an output LST amount and its SOL value, calculate the input SOL value by:

- binary search slab to obtain entries for input and output LST
- calculate total fee in nanos by adding `inp.input_fee_nanos` and `out.output_fee_nanos`
- calculate input LST's sol value using given `sol_value` of output lst assuming that the calculated fee was imposed to resulting input lst's SOL value

##### Data

| Name         | Value                          | Type |
| ------------ | ------------------------------ | ---- |
| discriminant | 1                              | u8   |
| amount       | amount of the output LST       | u64  |
| sol_value    | SOL value of amount output LST | u64  |

##### Return Data

| Name   | Value                          | Type |
| ------ | ------------------------------ | ---- |
| result | The calculated input SOL value | u64  |

##### Accounts

| Account             | Description            | Read/Write (R/W) | Signer (Y/N) |
| ------------------- | ---------------------- | ---------------- | ------------ |
| **input_lst_mint**  | Mint of the input LST  | R                | N            |
| **output_lst_mint** | Mint of the output LST | R                | N            |
| slab                | slab PDA               | R                | N            |

##### Procedure

#### PriceLpTokensToMint

Given an input LST amount and its SOL value, calculate the SOL value of the LP tokens to mint.

##### Data

| Name         | Value                         | Type |
| ------------ | ----------------------------- | ---- |
| discriminant | 2                             | u8   |
| amount       | amount of the input LST       | u64  |
| sol_value    | SOL value of amount input LST | u64  |

##### Return Data

| Name   | Value                                         | Type |
| ------ | --------------------------------------------- | ---- |
| result | The calculated SOL value of LP tokens to mint | u64  |

##### Accounts

| Account            | Description           | Read/Write (R/W) | Signer (Y/N) |
| ------------------ | --------------------- | ---------------- | ------------ |
| **input_lst_mint** | Mint of the input LST | R                | N            |
| slab               | slab PDA              | R                | N            |

##### Procedure

Call [PriceExactIn](#priceexactin) with `input_lst_mint=input_lst_mint`, `output_lst_mint=INF`

#### PriceLpTokensToRedeem

Given an input LP token amount and its SOL value, calculate the SOL value of the LST to redeem.

##### Data

| Name         | Value                        | Type |
| ------------ | ---------------------------- | ---- |
| discriminant | 3                            | u8   |
| amount       | amount of the input LP       | u64  |
| sol_value    | SOL value of amount input LP | u64  |

##### Return Data

| Name   | Value                                         | Type |
| ------ | --------------------------------------------- | ---- |
| result | The calculated SOL value of the LST to redeem | u64  |

##### Accounts

| Account             | Description            | Read/Write (R/W) | Signer (Y/N) |
| ------------------- | ---------------------- | ---------------- | ------------ |
| **output_lst_mint** | Mint of the output LST | R                | N            |
| slab                | slab PDA               | R                | N            |

##### Procedure

Call [PriceExactIn](#priceexactin) with `input_lst_mint=INF`, `output_lst_mint=output_lst_mint`

### Management Instructions

Only the current manager is authorized to execute.

#### Initialize

Permissionlessly initialize the program state. Can only be called once and sets manager to a hardcoded init manager.

##### Data

| Name         | Value | Type |
| ------------ | ----- | ---- |
| discriminant | 255   | u8   |

##### Accounts

| Account        | Description                            | Read/Write (R/W) | Signer (Y/N) |
| -------------- | -------------------------------------- | ---------------- | ------------ |
| payer          | Account paying for ProgramState's rent | W                | Y            |
| slab           | slab PDA                               | W                | N            |
| system_program | System program                         | R                | N            |

#### SetManager

Update the manager authority of the pricing program.

##### Data

| Name         | Value | Type |
| ------------ | ----- | ---- |
| discriminant | 254   | u8   |

##### Accounts

| Account         | Description                       | Read/Write (R/W) | Signer (Y/N) |
| --------------- | --------------------------------- | ---------------- | ------------ |
| current_manager | The current program manager       | R                | Y            |
| new_manager     | The new program manager to set to | R                | N            |
| slab            | slab PDA                          | W                | N            |

#### SetLstFee

Sets the lst fee for a mint. Adds a new entry onto slab if the mint does not already exist on it, otherwise, updates the existing entry.

##### Data

| Name             | Value                                                        | Type |
| ---------------- | ------------------------------------------------------------ | ---- |
| discriminant     | 253                                                          | u8   |
| input_fee_nanos  | fee in nanos to impose when the token type is used as input  | i32  |
| output_fee_nanos | fee in nanos to impose when the token type is used as output | i32  |

##### Accounts

| Account  | Description                                         | Read/Write (R/W) | Signer (Y/N) |
| -------- | --------------------------------------------------- | ---------------- | ------------ |
| manager  | The program manager                                 | R                | Y            |
| payer    | Account paying for additional slab's rent if needed | W                | Y            |
| slab     | slab PDA                                            | W                | N            |
| lst_mint | Mint of the LST to set fees for                     | R                | N            |

#### RemoveLst

Remove a LST's entry from the slab, resizing the slab account down.

##### Data

| Name         | Value | Type |
| ------------ | ----- | ---- |
| discriminant | 252   | u8   |

##### Accounts

| Account        | Description                   | Read/Write (R/W) | Signer (Y/N) |
| -------------- | ----------------------------- | ---------------- | ------------ |
| manager        | The program manager           | R                | Y            |
| refund_rent_to | Account to refund SOL rent to | W                | N            |
| slab           | slab PDA                      | W                | N            |
| lst_mint       | Mint of the LST to remove     | R                | N            |
