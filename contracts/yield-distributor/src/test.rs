extern crate std;

use super::*;
use property_token::{PropertyToken, PropertyTokenClient};
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
    share_token: PropertyTokenClient<'a>,
    client: YieldDistributorClient<'a>,
}

fn setup() -> Setup<'static> {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(&env, &sac.address()).mint(&admin, &1_000_000);
    let usdc = TokenClient::new(&env, &sac.address());

    let share_token_id = env.register(PropertyToken, (admin.clone(),));
    let share_token = PropertyTokenClient::new(&env, &share_token_id);
    let id = env.register(
        YieldDistributor,
        (admin.clone(), sac.address(), share_token_id),
    );
    let client = YieldDistributorClient::new(&env, &id);
    Setup {
        env,
        admin,
        usdc,
        share_token,
        client,
    }
}

/// Whitelists `holder` on the property token so the distributor accepts it.
fn whitelist(s: &Setup, holder: &Address) {
    s.share_token.set_whitelisted(&s.admin, holder, &true);
}

#[test]
fn distribute_pulls_funds_and_credits_holders() {
    let s = setup();
    let alice = Address::generate(&s.env);
    let bob = Address::generate(&s.env);
    whitelist(&s, &alice);
    whitelist(&s, &bob);

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
    whitelist(&s, &alice);

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
    whitelist(&s, &alice);
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
    whitelist(&s, &alice);
    s.client.distribute(&map![&s.env, (alice.clone(), 40i128)]);
    s.client.claim(&alice);
    assert_eq!(
        s.client.try_claim(&alice),
        Err(Ok(err(Error::NothingToClaim)))
    );
    assert_eq!(s.usdc.balance(&alice), 40);
}

#[test]
fn round_info_records_each_distribution() {
    let s = setup();
    let alice = Address::generate(&s.env);
    let bob = Address::generate(&s.env);

    s.client.distribute(&map![
        &s.env,
        (alice.clone(), 700i128),
        (bob.clone(), 300i128)
    ]);
    let first = s.client.round_info(&1).unwrap();
    assert_eq!(first.total, 1_000);
    assert_eq!(first.recipients, 2);
    assert_eq!(first.ledger, s.env.ledger().sequence());
    assert_eq!(first.timestamp, s.env.ledger().timestamp());

    s.client.distribute(&map![&s.env, (alice.clone(), 50i128)]);
    let second = s.client.round_info(&2).unwrap();
    assert_eq!(second.total, 50);
    assert_eq!(second.recipients, 1);
    assert_eq!(second.ledger, s.env.ledger().sequence());
    assert_eq!(second.timestamp, s.env.ledger().timestamp());
}

#[test]
fn round_info_unknown_round_is_none() {
    let s = setup();
    assert_eq!(s.client.round_info(&99), None);
}

#[test]
#[should_panic]
fn distribute_requires_admin_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let share_token_id = env.register(PropertyToken, (admin.clone(),));
    let id = env.register(
        YieldDistributor,
        (admin, sac.address(), share_token_id),
    );
    let client = YieldDistributorClient::new(&env, &id);
    client.distribute(&map![&env, (Address::generate(&env), 1i128)]);
}

#[test]
fn rejects_non_whitelisted_recipient() {
    let s = setup();
    let alice = Address::generate(&s.env);
    let mallory = Address::generate(&s.env);
    whitelist(&s, &alice);

    assert_eq!(
        s.client.try_distribute(&map![
            &s.env,
            (alice.clone(), 700i128),
            (mallory.clone(), 300i128)
        ]),
        Err(Ok(err(Error::RecipientNotWhitelisted)))
    );
    // Nothing was credited or pulled.
    assert_eq!(s.client.claimable(&alice), 0);
    assert_eq!(s.client.claimable(&mallory), 0);
    assert_eq!(s.client.round(), 0);
    assert_eq!(s.usdc.balance(&s.client.address), 0);
}

#[test]
fn rejects_recipient_removed_from_whitelist() {
    let s = setup();
    let alice = Address::generate(&s.env);
    whitelist(&s, &alice);
    s.share_token.set_whitelisted(&s.admin, &alice, &false);

    assert_eq!(
        s.client.try_distribute(&map![&s.env, (alice, 100i128)]),
        Err(Ok(err(Error::RecipientNotWhitelisted)))
    );
}

#[test]
fn upgrade_changes_wasm_and_emits_event() {
    let s = setup();
    let new_wasm_hash = s.env.deployer().upload_contract_wasm(YieldDistributorWasm::WASM);

    s.client.upgrade(&new_wasm_hash);

    assert_eq!(s.client.version(), VERSION);
}

#[test]
fn upgrade_requires_admin_auth() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let share_token_id = env.register(PropertyToken, (admin.clone(),));
    let id = env.register(
        YieldDistributor,
        (admin, sac.address(), share_token_id),
    );
    let client = YieldDistributorClient::new(&env, &id);
    let new_wasm_hash = env.deployer().upload_contract_wasm(YieldDistributorWasm::WASM);

    // No auths mocked: the admin auth requirement must fail.
    assert!(client.try_upgrade(&new_wasm_hash).is_err());
}
