Typescript tests for the ts sdk.

## Setup

- `pnpm install`
- Build onchain programs with `cargo-build-sbf`

## Run

Before running the tests:

- ensure `ts/sdk` rust crate has been rebuilt and reinstalled:
  ```sh
  pushd ../sdk
  make
  popd
  pnpm install
  ```
- rebuild the onchain programs if they have changed. The compiled `.so` files **MUST** be in `target/deploy/`

Then, start the local test validator with:

```sh
pnpm start:infra
```

Then, run the test script with:

```sh
pnpm test
```

After tests complete, teardown the local test validator with:

```sh
pnpm stop:infra
```

We do not use `package.json`'s `pretest` and `posttest` scripts for this because `posttest` does not run if tests failed and `test` exited with nonzero code.
