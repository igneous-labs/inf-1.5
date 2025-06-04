Typescript tests for the ts sdk.

## Setup

`pnpm install`

## Run

Before running the tests, make sure the `ts/sdk` rust crate has been rebuilt:

```sh
cd ../sdk
make
```

Then, run the test script with:

```sh
pnpm test
```

`pretest` and `posttest` scripts in `package.json` are responsible for spinning up and tearing down the local validator docker compose before and after running the tests.
