# CLAUDE.md

Guidance for Claude Code (and other agents) working in this repository.

## What this is

The Soroban smart contracts for defi-estate. The API in `defi-estate-backend` holds the admin key and signs admin actions (whitelist, mint, distribute); investors sign transfers and claims from `defi-estate-frontend` with a Stellar wallet. See `docs/contracts.md`.

## Non-negotiables

- **`make check` must pass** before work is called done (fmt, clippy with `-D warnings`, tests, wasm build).
- Every state-changing function checks auth (`require_auth`) and emits a `#[contractevent]`.
- Errors are typed `#[contracterror]` enums; never panic with a bare string.
- Arithmetic on balances uses `checked_*` and fails with `Error::Overflow`.
- Every new behavior gets a test in that crate's `src/test.rs`, including the failure paths.
- Build with `stellar contract build` or `--target wasm32v1-none`. Do not use `wasm32-unknown-unknown` on recent toolchains; it emits reference types the Soroban VM rejects.
- If a public function's signature changes, update `docs/contracts.md` and tell the frontend and backend repos.

## Commit conventions

Conventional Commits: `<type>(<scope>): <short lowercase summary>`. Scopes: `property-token`, `yield-distributor`, `ci`, `docs`.
