use soroban_sdk::{contracttype, Address, Env};

/// About 30 days at 5 second ledgers.
const TTL_EXTEND_TO: u32 = 518_400;
const TTL_THRESHOLD: u32 = TTL_EXTEND_TO - 17_280;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PropertyId,
    Name,
    Symbol,
    MaxSupply,
    TotalSupply,
    Balance(Address),
    Whitelisted(Address),
}

pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn total_supply(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

pub fn max_supply(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::MaxSupply).unwrap()
}

pub fn balance(env: &Env, id: &Address) -> i128 {
    let key = DataKey::Balance(id.clone());
    match env.storage().persistent().get(&key) {
        Some(b) => {
            env.storage()
                .persistent()
                .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
            b
        }
        None => 0,
    }
}

pub fn set_balance(env: &Env, id: &Address, amount: i128) {
    let key = DataKey::Balance(id.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn is_whitelisted(env: &Env, id: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::Whitelisted(id.clone()))
        .unwrap_or(false)
}

pub fn set_whitelisted(env: &Env, id: &Address, allowed: bool) {
    let key = DataKey::Whitelisted(id.clone());
    if allowed {
        env.storage().persistent().set(&key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
    } else {
        env.storage().persistent().remove(&key);
    }
}
