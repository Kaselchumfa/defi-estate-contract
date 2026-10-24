# defi-estate-contract

Soroban smart contracts for [defi-estate](https://github.com/defi-estate), a real estate tokenization and rental yield platform on Stellar. One `property-token` and one `yield-distributor` are deployed per property.

| Crate | Purpose |
| --- | --- |
| `contracts/property-token` | capped, whitelist-gated share token for one property (0 decimals) |
| `contracts/yield-distributor` | holds rental income in a payout asset (for example USDC) and lets holders claim it |

Full interface, errors and signing model: [docs/contracts.md](docs/contracts.md).

## Related repositories

- [defi-estate-frontend](https://github.com/defi-estate/defi-estate-frontend): Next.js webapp
- [defi-estate-backend](https://github.com/defi-estate/defi-estate-backend): Elysia API, holds the admin key that whitelists, mints and distributes

## Requirements

Rust stable with the `wasm32v1-none` target (`rustup target add wasm32v1-none`) and the [Stellar CLI](https://developers.stellar.org/docs/tools/cli).

## Commands

```bash
make check                 # fmt, clippy, test, wasm build (same as CI)
cargo test
stellar contract build     # or: cargo build --target wasm32v1-none --release
make fix                   # cargo fmt
```

## Deploy

```bash
stellar keys generate --global admin --network testnet --fund
./scripts/deploy-contracts.sh admin <property-uuid> "Lekki Garden Shares" LKGS 10000 <usdc-sac-id> testnet
```

Record the printed IDs in `deployments/<network>.json`.
