# Releasing

1. Bump the version in `Cargo.toml` and merge it to `main`.
2. Tag that commit and push the tag:

   ```sh
   git tag v0.1.1 && git push origin v0.1.1
   ```

3. The Publish workflow checks that the tag matches the version, runs the checks, and
   waits for a maintainer to approve the `release` environment. Approve it in the
   workflow run.

Published versions are immutable, and the repository blocks moving or deleting `v*`
tags. To fix a bad release, publish a new version.

## One-time setup

Publishing uses crates.io trusted publishing, so no token is stored. crates.io only
accepts a trusted publisher for a crate that already exists, so publish the first
version by hand with `cargo login` and `cargo publish`. Then, in the crate settings, add
a GitHub trusted publisher with owner `ternwire`, repository `ternwire-rust`, workflow
`publish.yml`, and environment `release`.
