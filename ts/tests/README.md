Typescript tests for the ts sdk.

## Setup

- Build onchain programs with `cargo-build-sbf`
- `pnpm install`

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

Then, run tests with

```sh
pnpm test
```

## Run With Independent Validator Process

Having a long-running test validator in the background that doesn't shutdown on test completion can be useful for debugging with explorer and other tools.

Start the local test validator with:

```sh
pnpm start:infra
```

Then, run the test script with:

```sh
pnpm vitest run
```

After tests complete, teardown the local test validator with:

```sh
pnpm stop:infra
```

We do not use `package.json`'s `pretest` and `posttest` scripts for this because `posttest` does not run if tests failed and `test` exited with nonzero code.
