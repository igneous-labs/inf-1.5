# inf1-test-utils

Common test utils. Includes things like

- common mollusk & `solana-*` interop
- `proptest Strategy`s for generating accounts

This library can be included in any library in here's `dev-dependencies` and used in integration tests, but may not be included under `dependencies`, else circular dependency.
