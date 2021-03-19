# Getting started

## Init a repos directory

Github/Bitbucket is detected from the git origin remote:
```
cargo run ~/path/to/repos init
```

init will prompt for a username/password that will be stored in a clear text `.prist/config.toml` in the repos directory.

## List commits from PR

```
cargo run ~/path/to/repos pr <pr-number>
```

## Debug output

```
RUST_LOG=DEBUG cargo run ~/path/to/repso/ pr <pr-number>
```
