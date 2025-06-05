# test-fixtures

## Clone Program

```sh
solana program dump <addr> <programs/out.so>
```

Then, add `--upgradeable-program <addr> <programs/out.so> none` to the docker compose file to add it to the local validator.

## Clone Account

```sh
solana account --output json -o <account.json> <addr>
```
