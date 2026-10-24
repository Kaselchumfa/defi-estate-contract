extern crate std;

use super::*;
use soroban_sdk::testutils::Address as _;

/// Contract functions panic with typed errors, so `try_` calls surface them as host errors.
fn err(e: Error) -> soroban_sdk::Error {
    soroban_sdk::Error::from_contract_error(e as u32)
}

struct Setup<'a> {
    env: Env,
    admin: Address,
    client: PropertyTokenClient<'a>,
}

fn setup() -> Setup<'static> {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register(
        PropertyToken,
        (
            admin.clone(),
            String::from_str(&env, "prop-001"),
            String::from_str(&env, "Lekki Garden Shares"),
            String::from_str(&env, "LKGS"),
            1_000i128,
        ),
    );
    let client = PropertyTokenClient::new(&env, &id);
    Setup { env, admin, client }
}

fn investor(s: &Setup) -> Address {
    let a = Address::generate(&s.env);
    s.client.set_whitelisted(&a, &true);
    a
}

#[test]
fn constructor_sets_metadata() {
    let s = setup();
    assert_eq!(s.client.admin(), s.admin);
    assert_eq!(s.client.symbol(), String::from_str(&s.env, "LKGS"));
    assert_eq!(s.client.max_supply(), 1_000);
    assert_eq!(s.client.total_supply(), 0);
    assert_eq!(s.client.decimals(), 0);
}

#[test]
fn mint_and_transfer_between_whitelisted_accounts() {
    let s = setup();
    let alice = investor(&s);
    let bob = investor(&s);

    s.client.mint(&alice, &600);
    s.client.transfer(&alice, &bob, &150);

    assert_eq!(s.client.balance(&alice), 450);
    assert_eq!(s.client.balance(&bob), 150);
    assert_eq!(s.client.total_supply(), 600);
}

#[test]
fn mint_requires_whitelist() {
    let s = setup();
    let stranger = Address::generate(&s.env);
    assert_eq!(
        s.client.try_mint(&stranger, &1),
        Err(Ok(err(Error::NotWhitelisted)))
    );
}

#[test]
fn mint_cannot_exceed_max_supply() {
    let s = setup();
    let alice = investor(&s);
    s.client.mint(&alice, &1_000);
    assert_eq!(
        s.client.try_mint(&alice, &1),
        Err(Ok(err(Error::MaxSupplyExceeded)))
    );
}

#[test]
fn transfer_to_non_whitelisted_fails() {
    let s = setup();
    let alice = investor(&s);
    let stranger = Address::generate(&s.env);
    s.client.mint(&alice, &10);
    assert_eq!(
        s.client.try_transfer(&alice, &stranger, &5),
        Err(Ok(err(Error::NotWhitelisted)))
    );
}

#[test]
fn transfer_more_than_balance_fails() {
    let s = setup();
    let alice = investor(&s);
    let bob = investor(&s);
    s.client.mint(&alice, &10);
    assert_eq!(
        s.client.try_transfer(&alice, &bob, &11),
        Err(Ok(err(Error::InsufficientBalance)))
    );
}

#[test]
fn rejects_non_positive_amounts() {
    let s = setup();
    let alice = investor(&s);
    assert_eq!(
        s.client.try_mint(&alice, &0),
        Err(Ok(err(Error::InvalidAmount)))
    );
}

#[test]
fn removing_from_whitelist_blocks_incoming_transfers() {
    let s = setup();
    let alice = investor(&s);
    let bob = investor(&s);
    s.client.mint(&alice, &10);
    s.client.set_whitelisted(&bob, &false);
    assert!(!s.client.is_whitelisted(&bob));
    assert_eq!(
        s.client.try_transfer(&alice, &bob, &1),
        Err(Ok(err(Error::NotWhitelisted)))
    );
}

#[test]
#[should_panic]
fn mint_requires_admin_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let id = env.register(
        PropertyToken,
        (
            admin,
            String::from_str(&env, "prop-001"),
            String::from_str(&env, "Shares"),
            String::from_str(&env, "SHR"),
            100i128,
        ),
    );
    let client = PropertyTokenClient::new(&env, &id);
    // No mocked auths: the admin has not signed, so this must panic.
    client.mint(&Address::generate(&env), &1);
}
