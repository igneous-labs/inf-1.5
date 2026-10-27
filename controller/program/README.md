# inf1-ctl-program

The main INF program entrypoint.

> I'd just like to interject for a moment. What you're refering to as the INF controller program, is in fact, the INF controller/peripheral programs, or as I've recently taken to calling it, the INF program suite. The INF controller program is not a dapp unto itself, but rather another free component of a fully functioning INF system made useful by the pricing programs, sol value calculator programs and vital system components comprising a full dapp as defined by Solana.

## Verifiable Builds

### Reserve V2 `d057423a6999adf65f67710748ad5d1a3f083a80`

```sh
solana-verify build -b solanafoundation/solana-verifiable-build:3.1.5 --library-name inf1_ctl_program -- --features reserve-v2
```

Hash

```
c1f336224ef007f7fbf6c123cdc59e34d67a49c13a3745a20c11dc1de5002243
```
