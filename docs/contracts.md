# Smart contracts

Rust/Soroban workspace in this repository (`soroban-sdk` 26). One `property-token` and one `yield-distributor` are deployed per property.

## property-token

Capped, whitelist-gated share token with 0 decimals.

| Function                                                                                                        | Auth     | Notes                                   |
| --------------------------------------------------------------------------------------------------------------- | -------- | --------------------------------------- |
| `__constructor(admin, property_id, name, symbol, max_supply)`                                                   | deployer | `max_supply` = number of shares         |
| `set_whitelisted(account, allowed)`                                                                             | admin    | KYC-reviewed holders only               |
| `mint(to, amount)`                                                                                              | admin    | `to` must be whitelisted; supply capped |
| `burn(from, amount)`                                                                                            | admin    | reduces holder balance and total supply |
| `transfer(from, to, amount)`                                                                                    | `from`   | `to` must be whitelisted                |
| `approve(from, spender, amount, expiration_ledger)`                                                             | `from`   | allowance expires at `expiration_ledger` |
| `transfer_from(spender, from, to, amount)`                                                                      | `spender`| `to` must be whitelisted                |
| `burn_from(spender, from, amount)`                                                                              | `spender`| spends allowance, reduces supply        |
| `pause()`                                                                                                       | admin    | blocks `mint` and `transfer`            |
| `unpause()`                                                                                                     | admin    |                                         |
| `propose_admin(new_admin)`                                                                                      | admin    | starts two-step admin transfer          |
| `accept_admin()`                                                                                                | new admin| completes admin transfer                |
| `cancel_admin_proposal()`                                                                                       | admin    | clears the pending admin                |
| `set_admin(new_admin)`                                                                                          | admin    |                                         |
| `upgrade(new_wasm_hash)`                                                                                        | admin    | in-place wasm upgrade                   |
| `balance`, `total_supply`, `max_supply`, `is_whitelisted`, `name`, `symbol`, `decimals`, `property_id`, `admin`, `allowance`, `is_paused`, `pending_admin` | none     | views                                   |
| `version`                                                                                                       | none     | crate version constant                  |

Errors: `InvalidAmount`, `NotWhitelisted`, `InsufficientBalance`, `MaxSupplyExceeded`, `Overflow`, `Paused`.

## yield-distributor

Holds rental income in a payout asset (for example a USDC Stellar Asset Contract) and lets holders claim it.

| Function                                                                                                        | Auth     | Notes                                                                   |
| --------------------------------------------------------------------------------------------------------------- | -------- | ----------------------------------------------------------------------- |
| `__constructor(admin, payout_token, share_token)`                                                               | deployer |                                                                         |
| `distribute(allocations: Map<Address, i128>)`                                                                   | admin    | pulls the sum from admin, credits each holder, max 100 entries per call |
| `claim(holder)`                                                                                                 | holder   | pays everything owed                                                    |
| `propose_admin(new_admin)`                                                                                      | admin    | starts two-step admin transfer                                          |
| `accept_admin()`                                                                                                | new admin| completes admin transfer                                                |
| `cancel_admin_proposal()`                                                                                       | admin    | clears the pending admin                                                |
| `upgrade(new_wasm_hash)`                                                                                        | admin    | in-place wasm upgrade                                                   |
| `claimable`, `round`, `round_info`, `total_distributed`, `total_claimed`, `payout_token`, `share_token`, `admin`, `pending_admin` | none     | views                                                                   |
| `version`                                                                                                       | none     | crate version constant                                                  |

Errors: `InvalidAmount`, `InvalidAllocations`, `NothingToClaim`, `RecipientNotWhitelisted`, `Overflow`.

Allocations are computed off-chain by the API from a balance snapshot, which keeps share transfers cheap and avoids per-transfer reward bookkeeping.

### Per-round audit records

Every `distribute` call stores a summary of the round so payouts can be verified on-chain without indexing events:

```rust
pub struct RoundInfo {
    pub total: i128,      // amount distributed in this round
    pub recipients: u32,  // number of allocations in the round
    pub ledger: u32,      // ledger sequence at distribution
    pub timestamp: u64,   // ledger timestamp at distribution
}
```

The record is written to persistent storage under `DataKey::Round(u32)` (the round number, matching the value returned by `round()`), and its TTL is extended like other persistent entries. `round_info(round)` returns `Some(RoundInfo)` for a past round and `None` for an unknown round.

## Build, test, deploy

```bash
cargo test
cargo clippy --all-targets -- -D warnings
stellar contract build            # targets wasm32v1-none

# with a funded identity:
stellar keys generate --global admin --network testnet --fund
scripts/deploy-contracts.sh admin <property-uuid> "Lekki Garden Shares" LKGS 10000 <usdc-sac-id> testnet
```

Then record the IDs in `deployments/testnet.json` and copy them into `src/shared/contracts/contracts.testnet.json` in both [defi-estate-frontend](https://github.com/defi-estate/defi-estate-frontend) and [defi-estate-backend](https://github.com/defi-estate/defi-estate-backend).

## Upgrading a deployed contract

Both contracts expose an admin-gated `upgrade(new_wasm_hash)` that calls `update_current_contract_wasm`, so a bug fix ships without migrating holders to a new contract ID. Storage (balances, whitelist, allocations, claims) is preserved.

1. Build the new wasm:

   ```bash
   stellar contract build            # targets wasm32v1-none
   ```

2. Upload the wasm and note the returned hash:

   ```bash
   stellar contract upload \
     --wasm target/wasm32v1-none/release/property_token.wasm \
     --source admin --network testnet
   ```

3. Invoke `upgrade` on the deployed contract with that hash (repeat for `yield_distributor.wasm` / the distributor ID):

   ```bash
   stellar contract invoke \
     --id <contract-id> \
     --source admin --network testnet \
     -- upgrade --new_wasm_hash <wasm-hash>
   ```

4. Verify the new code is live:

   ```bash
   stellar contract invoke --id <contract-id> --network testnet -- version
   ```

Notes:

- Only the contract admin can upgrade; the call fails auth otherwise.
- An `Upgraded` event carrying the new wasm hash is emitted on success.
- The new wasm must keep the existing storage layout and exported interface, or existing state becomes unreadable.
- Test the upgrade on testnet before mainnet, and record the new wasm hash alongside the contract IDs in `deployments/testnet.json`.
