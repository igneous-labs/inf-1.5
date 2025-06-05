# sanctum-token-ratio-compat

The math used in [`sanctum-token-ratio`](https://github.com/igneous-labs/sanctum-solana-utils/tree/master/sanctum-token-ratio) for reversing ratios and fees is slightly different from the new `sanctum-fee-ratio` and `sanctum-u64-ratio` libraries. This lib uses the old definitions so that the results given by the calc are equal to those onchain.

TODO: once we upgrade the sol value calculator programs to use the more correct new crates, we can delete this crate.
