# Release `malai`

- Do a version bump in `malai/Cargo.toml`.
- Update `MALAI_VERSION` variable in `malai.sh/install.sh` to the same value as
 `malai/Cargo.toml`.
- Update `CHANGELOG.md` to mention the new version that is about to be
  released.
- Go to github actions and run `release-malai.yml` workflow.
