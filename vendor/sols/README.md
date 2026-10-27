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
116c116
<             ));
---
>             ))
# changes to trailing semicolon formatting rules

$ diff -qr ./jiminy /path/to/sols/jiminy
# should return no diffs
```
