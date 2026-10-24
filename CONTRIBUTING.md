# Contributing

1. Fork the repository and branch from `develop`: `feat/<name>`, `fix/<name>`, `docs/<name>`.
2. One logical change per commit, using [Conventional Commits](https://www.conventionalcommits.org/).
3. Run `make check` and make sure it passes.
4. Open a pull request against `develop` using the PR template.

CI (`.github/workflows/contracts-ci.yml`) runs `cargo fmt --check`, clippy with `-D warnings`, `cargo test`, the wasm build with a 128 KiB size limit, and ShellCheck on `scripts/`.

Never commit secret keys or seed phrases. Report vulnerabilities privately to the maintainers.
