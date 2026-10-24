use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidAmount = 1,
    NotWhitelisted = 2,
    InsufficientBalance = 3,
    MaxSupplyExceeded = 4,
    Overflow = 5,
}
