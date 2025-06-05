Typescript tests for the ts sdk.

## Setup

`pnpm install`

## Run

Before running the tests, make sure the `ts/sdk` rust crate has been rebuilt:

```sh
cd ../sdk
make
```

Then, start the local test validator with:

```sh
docker compose -f ../../docker-compose-local-validator.yml up
```

Then, run the test script with:

```sh
pnpm test
```

After tests complete, teardown the local test validator with:

```sh
docker compose -f ../../docker-compose-local-validator.yml down
```

We do not use `package.json`'s `pretest` and `posttest` scripts for this because `posttest` does not run if tests failed and `test` exited with nonzero code.
