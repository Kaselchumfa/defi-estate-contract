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
