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
| `balance`, `total_supply`, `max_supply`, `is_whitelisted`, `name`, `symbol`, `decimals`, `property_id`, `admin`, `allowance`, `is_paused`, `pending_admin` | none     | views                                   |

Errors: `InvalidAmount`, `NotWhitelisted`, `InsufficientBalance`, `MaxSupplyExceeded`, `Overflow`, `Paused`.

## yield-distributor

Holds rental income in a payout asset (for example a USDC Stellar Asset Contract) and lets holders claim it.

| Function                                                                                           | Auth     | Notes                                                                   |
| -------------------------------------------------------------------------------------------------- | -------- | ----------------------------------------------------------------------- |
| `__constructor(admin, payout_token, share_token)`                                                  | deployer |                                                                         |
| `distribute(allocations: Map<Address, i128>)`                                                      | admin    | pulls the sum from admin, credits each holder, max 100 entries per call |
| `claim(holder)`                                                                                    | holder   | pays everything owed                                                    |
| `propose_admin(new_admin)`                                                                         | admin    | starts two-step admin transfer                                          |
| `accept_admin()`                                                                                   | new admin| completes admin transfer                                                |
| `cancel_admin_proposal()`                                                                          | admin    | clears the pending admin                                                |
| `claimable`, `round`, `total_distributed`, `total_claimed`, `payout_token`, `share_token`, `admin`, `pending_admin` | none     | views                                                                   |

Errors: `InvalidAmount`, `InvalidAllocations`, `NothingToClaim`, `Overflow`.

Allocations are computed off-chain by the API from a balance snapshot, which keeps share transfers cheap and avoids per-transfer reward bookkeeping.

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
