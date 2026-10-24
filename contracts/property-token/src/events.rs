use soroban_sdk::{contractevent, Address, BytesN, Env};

#[contractevent]
pub struct Mint {
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
pub struct Transfer {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
pub struct WhitelistUpdated {
    #[topic]
    pub account: Address,
    pub allowed: bool,
}

#[contractevent]
pub struct AdminChanged {
    pub new_admin: Address,
}

#[contractevent]
pub struct Upgraded {
    pub new_wasm_hash: BytesN<32>,
}

pub fn mint(env: &Env, to: Address, amount: i128) {
    Mint { to, amount }.publish(env);
}

pub fn transfer(env: &Env, from: Address, to: Address, amount: i128) {
    Transfer { from, to, amount }.publish(env);
}

pub fn whitelist_updated(env: &Env, account: Address, allowed: bool) {
    WhitelistUpdated { account, allowed }.publish(env);
}

pub fn admin_changed(env: &Env, new_admin: Address) {
    AdminChanged { new_admin }.publish(env);
}

pub fn upgraded(env: &Env, new_wasm_hash: BytesN<32>) {
    Upgraded { new_wasm_hash }.publish(env);
}