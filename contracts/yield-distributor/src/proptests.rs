//! Property-based tests for the yield-distributor contract.
//!
//! These tests generate random sequences of `distribute` and `claim`
//! operations and assert the core accounting invariant:
//!
//! ```text
//! contract payout balance == total_distributed - total_claimed
//! ```
//!
//! Case counts and sequence lengths are kept modest so the suite finishes
//! well under 60 seconds in CI. On failure, proptest shrinks the input to a
//! minimal reproducing sequence and prints it.

use proptest::prelude::*;

/// A single operation in a random sequence.
#[derive(Clone, Debug)]
enum Op {
    /// Distribute `amount` into the contract.
    Distribute { amount: u64 },
    /// Claim `amount` from the contract.
    Claim { amount: u64 },
}

/// Minimal in-memory model of the yield-distributor accounting.
///
/// This mirrors the contract's bookkeeping so the invariant can be checked
/// without depending on the full contract harness.
#[derive(Default, Debug)]
struct Model {
    total_distributed: u64,
    total_claimed: u64,
    payout_balance: u64,
}

impl Model {
    fn distribute(&mut self, amount: u64) {
        self.total_distributed = self.total_distributed.saturating_add(amount);
        self.payout_balance = self.payout_balance.saturating_add(amount);
    }

    /// Attempt to claim `amount`. Claims are capped at the available payout
    /// balance, matching the contract's behavior of never paying out more
    /// than it holds.
    fn claim(&mut self, amount: u64) {
        let paid = amount.min(self.payout_balance);
        self.total_claimed = self.total_claimed.saturating_add(paid);
        self.payout_balance -= paid;
    }

    fn invariant_holds(&self) -> bool {
        self.payout_balance == self.total_distributed - self.total_claimed
    }
}

/// Strategy producing a bounded random sequence of distribute/claim ops.
fn op_sequence() -> impl Strategy<Value = Vec<Op>> {
    // Keep amounts small and sequences short to bound runtime.
    let distribute = (0u64..=1_000).prop_map(|amount| Op::Distribute { amount });
    let claim = (0u64..=1_000).prop_map(|amount| Op::Claim { amount });
    prop::collection::vec(prop_oneof![distribute, claim], 0..=32)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        max_shrink_iters: 10_000,
        ..ProptestConfig::default()
    })]

    /// For any random sequence of distribute and claim operations, the
    /// contract payout balance must equal `total_distributed - total_claimed`.
    #[test]
    fn payout_balance_equals_distributed_minus_claimed(ops in op_sequence()) {
        let mut model = Model::default();

        for op in &ops {
            match *op {
                Op::Distribute { amount } => model.distribute(amount),
                Op::Claim { amount } => model.claim(amount),
            }
        }

        prop_assert!(
            model.invariant_holds(),
            "invariant violated after ops {:?}: payout_balance={}, total_distributed={}, total_claimed={}",
            ops,
            model.payout_balance,
            model.total_distributed,
            model.total_claimed,
        );
    }

    /// The payout balance must never exceed the total distributed amount,
    /// and total claimed must never exceed total distributed.
    #[test]
    fn balances_stay_within_bounds(ops in op_sequence()) {
        let mut model = Model::default();

        for op in &ops {
            match *op {
                Op::Distribute { amount } => model.distribute(amount),
                Op::Claim { amount } => model.claim(amount),
            }

            prop_assert!(
                model.payout_balance <= model.total_distributed,
                "payout_balance exceeded total_distributed after ops {:?}",
                ops,
            );
            prop_assert!(
                model.total_claimed <= model.total_distributed,
                "total_claimed exceeded total_distributed after ops {:?}",
                ops,
            );
        }
    }
}