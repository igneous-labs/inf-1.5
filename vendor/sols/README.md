# sols

sols crates vendored from upstream.

TODO: delete this and use upstream when sols repo is open-sourced

## Commit

```
97ab052023685b100443ba8c8bad002b52a9dc80
```

## Verify Integrity

```sh
$ diff -r ./core /path/to/sols/core
Only in ./core/: rustfmt.toml
diff -r ./core/src/rebal/claim_holding.rs /home/dhy/sanctum/sols/sols/core/src/rebal/claim_holding.rs
116c116
<             ));
---
>             ))
# rustfmt.toml diff and changes to trailing semicolon formatting rules

$ diff -qr ./jiminy /path/to/sols/jiminy
Only in ./jiminy: rustfmt.toml
# should be the only diff
```
