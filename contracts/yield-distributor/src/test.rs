extern crate std;

use super::*;
use soroban_sdk::map;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};

/// Contract functions panic with typed errors, so `try_` calls surface them as host errors.
fn err(e: Error) -> soroban_sdk::Error {
    soroban_sdk::Error::from_contract_error(e as u32)
}

struct Setup<'a> {
    env: Env,
    admin: Address,
    usdc: TokenClient<'a>,
    client: YieldDistributorClient<'a>,
}

fn setup() -> Setup<'static> {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(&env, &sac.address()).mint(&admin, &1_000_000);
    let usdc = TokenClient::new(&env, &sac.address());

    let share_token = Address::generate(&env);
    let id = env.register(
        YieldDistributor,
        (admin.clone(), sac.address(), share_token),
    );
    let client = YieldDistributorClient::new(&env, &id);
    Setup {
        env,
        admin,
        usdc,
        client,
    }
}

#[test]
fn distribute_pulls_funds_and_credits_holders() {
    let s = setup();
    let alice = Address::generate(&s.env);
    let bob = Address::generate(&s.env);

    let round = s.client.distribute(&map![
        &s.env,
        (alice.clone(), 700i128),
        (bob.clone(), 300i128)
    ]);

    assert_eq!(round, 1);
    assert_eq!(s.client.claimable(&alice), 700);
    assert_eq!(s.client.claimable(&bob), 300);
    assert_eq!(s.client.total_distributed(), 1_000);
    assert_eq!(s.usdc.balance(&s.client.address), 1_000);
    assert_eq!(s.usdc.balance(&s.admin), 999_000);
}

#[test]
fn claims_accumulate_across_rounds() {
    let s = setup();
    let alice = Address::generate(&s.env);

    s.client.distribute(&map![&s.env, (alice.clone(), 100i128)]);
    s.client.distribute(&map![&s.env, (alice.clone(), 50i128)]);
    assert_eq!(s.client.round(), 2);

    assert_eq!(s.client.claim(&alice), 150);
    assert_eq!(s.usdc.balance(&alice), 150);
    assert_eq!(s.client.claimable(&alice), 0);
    assert_eq!(s.client.total_claimed(), 150);
}

#[test]
fn claim_with_nothing_owed_fails() {
    let s = setup();
    let alice = Address::generate(&s.env);
    assert_eq!(
        s.client.try_claim(&alice),
        Err(Ok(err(Error::NothingToClaim)))
    );
}

#[test]
fn rejects_empty_and_non_positive_allocations() {
    let s = setup();
    let alice = Address::generate(&s.env);
    assert_eq!(
        s.client.try_distribute(&Map::new(&s.env)),
        Err(Ok(err(Error::InvalidAllocations)))
    );
    assert_eq!(
        s.client.try_distribute(&map![&s.env, (alice, 0i128)]),
        Err(Ok(err(Error::InvalidAmount)))
    );
}

#[test]
fn double_claim_pays_once() {
    let s = setup();
    let alice = Address::generate(&s.env);
    s.client.distribute(&map![&s.env, (alice.clone(), 40i128)]);
    s.client.claim(&alice);
    assert_eq!(
        s.client.try_claim(&alice),
        Err(Ok(err(Error::NothingToClaim)))
    );
    assert_eq!(s.usdc.balance(&alice), 40);
}

#[test]
#[should_panic]
fn distribute_requires_admin_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let id = env.register(
        YieldDistributor,
        (admin, sac.address(), Address::generate(&env)),
    );
    let client = YieldDistributorClient::new(&env, &id);
    client.distribute(&map![&env, (Address::generate(&env), 1i128)]);
}
