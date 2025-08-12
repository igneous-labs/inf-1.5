# `FlatSlab` Pricing Program

This is basically the same as the [flatfee pricing program](../flatfee/), but with the following changes:

- Instead of `i16` bps, we use a `i32` with `1_000_000_000` as the denominator to calculate rate (instead of `10_000` for bps previously) for more granular control.
- All input and output fees are stored in the same static PDA in an array of `(mint, input_fee, output_fee)` sorted by mint i.e. a giant slab. Binary searched are performed to read the fees to price trades for each mint.
- Instead of special-casing `PriceLpTokensToMint` and `PriceLpTokensToRedeem`, the LP token (INF) is simply treated as another mint on the slab. Identity of this LP token mint is hardcoded into the program.
- This slab account also contains a header of a `manager` pubkey that specifies who is authorized to
  - set new `manager`
  - set fees for each mint
  - add and remove mints from the slab
