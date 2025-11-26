# inf1-test-utils

Common test utils. Includes things like

- common mollusk & `solana-*` interop
- `proptest Strategy`s for generating accounts

## Circular Dependencies

This library can be included in any library in here's `dev-dependencies`, but may not be included under `dependencies`, else circular dependency.

When used for unit tests, things that use types defined in the other library cannot be used.

```
= note: expected function signature `fn((_, _, typedefs::rps::Rps, inf1_ctl_core::typedefs::fee_nanos::FeeNanos)) -> _`
            found function signature `fn((_, _, typedefs::rps::Rps, typedefs::fee_nanos::FeeNanos)) -> _`
```

No restrictions apply for integration tests.
