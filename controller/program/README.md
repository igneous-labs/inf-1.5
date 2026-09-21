# inf1-ctl-program

The main INF program entrypoint.

> I'd just like to interject for a moment. What you're refering to as the INF controller program, is in fact, the INF controller/peripheral programs, or as I've recently taken to calling it, the INF program suite. The INF controller program is not a dapp unto itself, but rather another free component of a fully functioning INF system made useful by the pricing programs, sol value calculator programs and vital system components comprising a full dapp as defined by Solana.

## Verifiable Builds

### Reserve V2 `926390acd2f6f4d7cdeb9805aaecbefe7c9a0827`

```sh
solana-verify build -b solanafoundation/solana-verifiable-build:3.1.5 --library-name inf1_ctl_program -- --features reserve-v2
```

Hash

```
0fd4efecbddad4afe316fd37f906b7caf1e327a710ccd3708d9b6c201a99b999
```
