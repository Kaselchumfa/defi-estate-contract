//! Property-based tests for the property-token contract.
//!
//! These tests exercise the core accounting invariants across random
//! sequences of `mint` and `transfer` operations:
//!
//! * `total_supply` always equals the sum of all balances.
//! * `total_supply` never exceeds `max_supply`.
//!
//! Case counts and sequence lengths are kept modest so the suite finishes
//! well under 60 seconds in CI. `proptest` shrinks failing cases to a
//! minimal reproducing sequence automatically.

use proptest::prelude::*;

use crate::{PropertyToken, PropertyTokenClient};

/// A single operation applied to the token contract.
#[derive(Clone, Debug)]
enum Op {
    /// Mint `amount` tokens to account `to`.
    Mint { to: usize, amount: u64 },
    /// Transfer `amount` tokens from account `from` to account `to`.
    Transfer { from: usize, to: usize, amount: u64 },
}

/// Number of distinct accounts used in the generated scenarios.
const NUM_ACCOUNTS: usize = 4;

/// Upper bound for a single mint amount.
const MAX_MINT: u64 = 1_000;

/// Upper bound for a single transfer amount.
const MAX_TRANSFER: u64 = 500;

/// Maximum number of operations in a generated sequence.
const MAX_OPS: usize = 24;

/// Strategy producing a single operation.
fn op_strategy() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0..NUM_ACCOUNTS, 0..=MAX_MINT).prop_map(|(to, amount)| Op::Mint { to, amount }),
        (0..NUM_ACCOUNTS, 0..NUM_ACCOUNTS, 0..=MAX_TRANSFER)
            .prop_map(|(from, to, amount)| Op::Transfer { from, to, amount }),
    ]
}

/// Strategy producing a bounded sequence of operations.
fn ops_strategy() -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec(op_strategy(), 0..=MAX_OPS)
}

/// Sum of all balances held by the tracked accounts.
fn sum_balances(client: &PropertyTokenClient, accounts: &[soroban_sdk::Address]) -> u64 {
    accounts
        .iter()
        .map(|addr| client.balance(addr))
        .fold(0u64, |acc, bal| acc.saturating_add(bal))
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        max_shrink_iters: 2_000,
        ..ProptestConfig::default()
    })]

    /// For random mint and transfer sequences, `total_supply` equals the sum
    /// of balances and never exceeds `max_supply`.
    #[test]
    fn supply_equals_sum_of_balances_and_respects_cap(ops in ops_strategy()) {
        let env = soroban_sdk::Env::default();
        env.mock_all_auths();

        let admin = soroban_sdk::Address::generate(&env);
        let max_supply: u64 = (MAX_OPS as u64) * MAX_MINT;

        let contract_id = env.register(PropertyToken, ());
        let client = PropertyTokenClient::new(&env, &contract_id);

        client.initialize(&admin, &max_supply);

        let accounts: Vec<soroban_sdk::Address> = (0..NUM_ACCOUNTS)
            .map(|_| soroban_sdk::Address::generate(&env))
            .collect();

        for op in ops {
            match op {
                Op::Mint { to, amount } => {
                    let recipient = &accounts[to];
                    // Minting may legitimately fail once the cap is reached;
                    // the invariants below must still hold afterwards.
                    let _ = client.try_mint(&admin, recipient, &amount);
                }
                Op::Transfer { from, to, amount } => {
                    let sender = &accounts[from];
                    let recipient = &accounts[to];
                    // Transfers may fail for insufficient balance; ignore the
                    // result and keep checking invariants.
                    let _ = client.try_transfer(sender, recipient, &amount);
                }
            }

            let total_supply = client.total_supply();
            let balances = sum_balances(&client, &accounts);

            prop_assert_eq!(total_supply, balances);
            prop_assert!(total_supply <= max_supply);
        }

        // Final invariant check after the whole sequence.
        let total_supply = client.total_supply();
        let balances = sum_balances(&client, &accounts);
        prop_assert_eq!(total_supply, balances);
        prop_assert!(total_supply <= max_supply);
    }
}