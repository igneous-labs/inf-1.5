# stake-pools

stake pool program binaries.

## Notes

Even though we dont call any of the stake pool programs, we have to clone the stake pool program binaries instead of adding program + program data as test fixtures json account because ever since solana-sdk v1.17, adding accounts owned by any of the bpf loader programs to `solana-test-validator` will result in an error (`ProgramCacheHitMaxLimit` for vers 2.1.X).

As such, we clone the binaries, load them in with `--upgradeable-program`, and edit all generic sol value calculator program state data to set `last_upgrade_slot=0`.
