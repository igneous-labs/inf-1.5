# org

Notes on organization of files and modules.

## `*-core` vs `*-std`

`*-core` libraries must be `no-std` and (almost) dependency-free. It should contain the base typedefs and functionality. Should operate on and return primitive values as much as possible.

`*-std` libraries build on top of the `*-core` libraries and can use the rust std lib. It should re-export `*-core::*` so that consumers should only need to either import the `*-core` lib or `*-std` lib.

`get_accounts_to_update()` + `update_with_fetched_accounts()` functionality should be placed in the `*-std` lib because it brings in an additional `inf1-update-traits` dependency.
