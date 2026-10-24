//! Rental yield distributor.
//!
//! The admin (the API's signing account) computes each holder's share of a
//! rental income period off-chain from a property-token balance snapshot and
//! calls `distribute` with those allocations. The payout asset (for example a
//! USDC Stellar Asset Contract) is pulled from the admin into this contract in
//! the same call, and holders withdraw their balance with `claim`.
#![no_std]

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod test;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, Env, Map};
use storage::DataKey;

pub use storage::RoundInfo;

/// Upper bound on allocations per call, to stay well inside the resource limits.
pub const MAX_ALLOCATIONS: u32 = 100;

#[contract]
pub struct YieldDistributor;

#[contractimpl]
impl YieldDistributor {
    pub fn __constructor(env: Env, admin: Address, payout_token: Address, share_token: Address) {
        let s = env.storage().instance();
        s.set(&DataKey::Admin, &admin);
        s.set(&DataKey::PayoutToken, &payout_token);
        s.set(&DataKey::ShareToken, &share_token);
        s.set(&DataKey::Round, &0u32);
        s.set(&DataKey::TotalDistributed, &0i128);
        s.set(&DataKey::TotalClaimed, &0i128);
    }

    /// Credits each holder with their allocation and pulls the total from the admin.
    /// Returns the new distribution round number.
    pub fn distribute(env: Env, allocations: Map<Address, i128>) -> u32 {
        let admin = storage::admin(&env);
        admin.require_auth();

        if allocations.is_empty() || allocations.len() > MAX_ALLOCATIONS {
            panic_with_error!(&env, Error::InvalidAllocations);
        }

        let mut total: i128 = 0;
        for (holder, amount) in allocations.iter() {
            if amount <= 0 {
                panic_with_error!(&env, Error::InvalidAmount);
            }
            total = total
                .checked_add(amount)
                .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
            let next = storage::claimable(&env, &holder)
                .checked_add(amount)
                .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
            storage::set_claimable(&env, &holder, next);
        }

        token::Client::new(&env, &storage::payout_token(&env)).transfer(
            &admin,
            env.current_contract_address(),
            &total,
        );

        let round = storage::bump_round(&env);
        storage::add_total(&env, DataKey::TotalDistributed, total);
        let info = RoundInfo {
            total,
            recipients: allocations.len(),
            ledger: env.ledger().sequence(),
            timestamp: env.ledger().timestamp(),
        };
        storage::set_round_info(&env, round, &info);
        storage::extend_instance(&env);
        events::distributed(&env, round, total, allocations.len());
        round
    }

    /// Sends the holder everything credited to them so far. Returns the amount paid.
    pub fn claim(env: Env, holder: Address) -> i128 {
        holder.require_auth();
        let amount = storage::claimable(&env, &holder);
        if amount == 0 {
            panic_with_error!(&env, Error::NothingToClaim);
        }
        storage::set_claimable(&env, &holder, 0);
        storage::add_total(&env, DataKey::TotalClaimed, amount);

        token::Client::new(&env, &storage::payout_token(&env)).transfer(
            &env.current_contract_address(),
            &holder,
            &amount,
        );
        events::claimed(&env, holder, amount);
        amount
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        storage::admin(&env).require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    // Views

    pub fn claimable(env: Env, holder: Address) -> i128 {
        storage::claimable(&env, &holder)
    }

    pub fn round(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Round).unwrap_or(0)
    }

    pub fn round_info(env: Env, round: u32) -> Option<RoundInfo> {
        storage::round_info(&env, round)
    }

    pub fn total_distributed(env: Env) -> i128 {
        storage::total(&env, DataKey::TotalDistributed)
    }

    pub fn total_claimed(env: Env) -> i128 {
        storage::total(&env, DataKey::TotalClaimed)
    }

    pub fn payout_token(env: Env) -> Address {
        storage::payout_token(&env)
    }

    pub fn share_token(env: Env) -> Address {
        env.storage().instance().get(&DataKey::ShareToken).unwrap()
    }

    pub fn admin(env: Env) -> Address {
        storage::admin(&env)
    }
}
