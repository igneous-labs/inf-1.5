# inf1-svc-generic

Defines common behaviours of generic sol value calculator programs. These programs:

- Have a single `pda("state")` program account that:
  - Records the last upgrade slot of the stake pool program
  - Records the sol value calculator program's manager
- Have the same interfaces for all instructions (both the SOL value calculator interface and admin-facing instructions)
  - https://github.com/igneous-labs/S/blob/master/docs/sol-value-calculator-programs/generic_pool.md
  - Computes `LstToSol` and `SolToLst` using `pool_state` account + mint account data

TODO: this crate should be renamed to `inf1-svc-generic-core` to follow the same convention.
