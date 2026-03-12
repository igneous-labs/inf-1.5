# test-fixtures

Notes on structures and changes to test fixtures accounts.

## Manual Account Changes

### `pool-state`

- pricing program changed from `flatfee` to `flatslab`

### `lst-state-list`

- Removed all mints except the ones we have test-fixtures accounts for
  - order is [wsol, stsol, msol, jupsol]
- Re-enabled stsol input

### SOL Value Calculator Programs ProgramState accounts

- Set `last-update-slot` to 0
