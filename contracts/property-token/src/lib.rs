//! Property share token.
//!
//! One instance per property. The supply is capped at the number of shares the
//! property was split into, only the admin can mint, and shares can only be held
//! by whitelisted (KYC-reviewed) accounts. Shares are whole units (0 decimals).
#![no_std]

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod test;

pub use errors::Error;

use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, String};
use storage::DataKey;

#[contract]
pub struct PropertyToken;

#[contractimpl]
impl PropertyToken {
    /// Sets the admin, token metadata and the maximum number of shares.
    /// `property_id` is the off-chain identifier (the API's property UUID).
    pub fn __constructor(
        env: Env,
        admin: Address,
        property_id: String,
        name: String,
        symbol: String,
        max_supply: i128,
    ) {
        if max_supply <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        let s = env.storage().instance();
        s.set(&DataKey::Admin, &admin);
        s.set(&DataKey::PropertyId, &property_id);
        s.set(&DataKey::Name, &name);
        s.set(&DataKey::Symbol, &symbol);
        s.set(&DataKey::MaxSupply, &max_supply);
        s.set(&DataKey::TotalSupply, &0i128);
    }

    // Admin

    pub fn set_whitelisted(env: Env, account: Address, allowed: bool) {
        storage::admin(&env).require_auth();
        storage::set_whitelisted(&env, &account, allowed);
        events::whitelist_updated(&env, account, allowed);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        storage::admin(&env).require_auth();
        require_positive(&env, amount);
        require_whitelisted(&env, &to);

        let supply = storage::total_supply(&env)
            .checked_add(amount)
            .unwrap_or_else(|| panic_with_error!(&env, Error::Overflow));
        if supply > storage::max_supply(&env) {
            panic_with_error!(&env, Error::MaxSupplyExceeded);
        }
        env.storage().instance().set(&DataKey::TotalSupply, &supply);
        storage::set_balance(&env, &to, storage::balance(&env, &to) + amount);
        storage::extend_instance(&env);
        events::mint(&env, to, amount);
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        storage::admin(&env).require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        events::admin_changed(&env, new_admin);
    }

    // Holders

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        require_positive(&env, amount);
        require_whitelisted(&env, &to);

        let from_balance = storage::balance(&env, &from);
        if from_balance < amount {
            panic_with_error!(&env, Error::InsufficientBalance);
        }
        storage::set_balance(&env, &from, from_balance - amount);
        storage::set_balance(&env, &to, storage::balance(&env, &to) + amount);
        events::transfer(&env, from, to, amount);
    }

    // Views

    pub fn balance(env: Env, id: Address) -> i128 {
        storage::balance(&env, &id)
    }

    pub fn total_supply(env: Env) -> i128 {
        storage::total_supply(&env)
    }

    pub fn max_supply(env: Env) -> i128 {
        storage::max_supply(&env)
    }

    pub fn is_whitelisted(env: Env, account: Address) -> bool {
        storage::is_whitelisted(&env, &account)
    }

    pub fn admin(env: Env) -> Address {
        storage::admin(&env)
    }

    pub fn property_id(env: Env) -> String {
        env.storage().instance().get(&DataKey::PropertyId).unwrap()
    }

    pub fn name(env: Env) -> String {
        env.storage().instance().get(&DataKey::Name).unwrap()
    }

    pub fn symbol(env: Env) -> String {
        env.storage().instance().get(&DataKey::Symbol).unwrap()
    }

    pub fn decimals(_env: Env) -> u32 {
        0
    }
}

fn require_positive(env: &Env, amount: i128) {
    if amount <= 0 {
        panic_with_error!(env, Error::InvalidAmount);
    }
}

fn require_whitelisted(env: &Env, account: &Address) {
    if !storage::is_whitelisted(env, account) {
        panic_with_error!(env, Error::NotWhitelisted);
    }
}
