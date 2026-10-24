use soroban_sdk::{contractevent, Address, Env};

#[contractevent]
pub struct Distributed {
    #[topic]
    pub round: u32,
    pub total: i128,
    pub recipients: u32,
}

#[contractevent]
pub struct Claimed {
    #[topic]
    pub holder: Address,
    pub amount: i128,
}

#[contractevent]
pub struct AdminProposed {
    #[topic]
    pub new_admin: Address,
}

#[contractevent]
pub struct AdminChanged {
    #[topic]
    pub new_admin: Address,
}

pub fn distributed(env: &Env, round: u32, total: i128, recipients: u32) {
    Distributed {
        round,
        total,
        recipients,
    }
    .publish(env);
}

pub fn claimed(env: &Env, holder: Address, amount: i128) {
    Claimed { holder, amount }.publish(env);
}

pub fn admin_proposed(env: &Env, new_admin: Address) {
    AdminProposed { new_admin }.publish(env);
}

pub fn admin_changed(env: &Env, new_admin: Address) {
    AdminChanged { new_admin }.publish(env);
}