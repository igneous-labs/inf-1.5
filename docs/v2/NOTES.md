# Notes

Misc notes about edge cases etc

## Circularity

INF itself or any LST that is transitively backed by INF (e.g. a LST that holds another LST that holds INF) must never be added to the pool. Otherwise this will cause unbounded minting of INF tokens and the bricking of the pool.

## Token-22

The following token-2022 mint extensions are not supported for constituent LSTs:

- TransferHook
- TransferFee

The program does not explicitly enforce this. The `admin` is reponsible for verifying that all tokens added to the pool do not have these extensions.
