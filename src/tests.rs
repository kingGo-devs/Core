use soroban_sdk::{testutils::Address as _, token, Address, Bytes, Env};

use crate::{ProposalState, VotingContract, VotingContractClient};

fn setup_test_env() -> (Env, VotingContractClient, Address, Address, u32) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract(admin.clone());
    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&user, &5000i128);

    let contract_id = env.register_contract(None, VotingContract);
    let client = VotingContractClient::new(&env, &contract_id);

    client.initialize(&admin, &token_address, &1000i128);

    (env, client, admin, user, token_address)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    let contract_id = env.register_contract(None, VotingContract);
    let client = VotingContractClient::new(&env, &contract_id);

    client.initialize(&admin, &token_address, &1000i128);

    assert_eq!(client.get_proposal_count(), 0u32);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    let contract_id = env.register_contract(None, VotingContract);
    let client = VotingContractClient::new(&env, &contract_id);

    client.initialize(&admin, &token_address, &1000i128);
    client.initialize(&admin, &token_address, &2000i128);
}

#[test]
fn test_create_proposal_sufficient_balance() {
    let (_env, client, _admin, user, _token) = setup_test_env();

    let desc_hash =
        Bytes::from_slice(&_env, b"QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco");
    let expiration = _env.ledger().sequence() + 1000;

    let proposal_id = client.initialize_proposal(&user, &desc_hash, &expiration);

    assert_eq!(proposal_id, 1u32);
    assert_eq!(client.get_proposal_count(), 1u32);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.state, ProposalState::Active);
    assert_eq!(proposal.description_hash, desc_hash);
    assert_eq!(proposal.total_yes_votes, 0i128);
    assert_eq!(proposal.total_no_votes, 0i128);
    assert_eq!(proposal.expiration_ledger, expiration);
    assert_eq!(proposal.creator, user);
}

#[test]
#[should_panic(expected = "insufficient token balance to create proposal")]
fn test_create_proposal_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let poor_user = Address::generate(&env);

    let token_address = env.register_stellar_asset_contract(admin.clone());
    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&poor_user, &500i128);

    let contract_id = env.register_contract(None, VotingContract);
    let client = VotingContractClient::new(&env, &contract_id);

    client.initialize(&admin, &token_address, &1000i128);

    let desc_hash = Bytes::from_slice(&env, b"some hash");
    let expiration = env.ledger().sequence() + 1000;
    client.initialize_proposal(&poor_user, &desc_hash, &expiration);
}

#[test]
fn test_vote_yes() {
    let (_env, client, _admin, user, _token) = setup_test_env();

    let desc_hash = Bytes::from_slice(&_env, b"proposal A");
    let expiration = _env.ledger().sequence() + 1000;
    let proposal_id = client.initialize_proposal(&user, &desc_hash, &expiration);

    client.vote(&user, &proposal_id, &true);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.total_yes_votes, 5000i128);
    assert_eq!(proposal.total_no_votes, 0i128);
    assert!(client.has_voted(&user, &proposal_id));
}

#[test]
fn test_vote_no() {
    let (_env, client, _admin, user, _token) = setup_test_env();

    let desc_hash = Bytes::from_slice(&_env, b"proposal B");
    let expiration = _env.ledger().sequence() + 1000;
    let proposal_id = client.initialize_proposal(&user, &desc_hash, &expiration);

    client.vote(&user, &proposal_id, &false);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.total_yes_votes, 0i128);
    assert_eq!(proposal.total_no_votes, 5000i128);
}

#[test]
fn test_multiple_voters() {
    let (_env, client, admin, _user, token_address) = setup_test_env();

    let voter1 = Address::generate(&_env);
    let voter2 = Address::generate(&_env);

    let sac = token::StellarAssetClient::new(&_env, &token_address);
    sac.mint(&voter1, &3000i128);
    sac.mint(&voter2, &2000i128);

    let desc_hash = Bytes::from_slice(&_env, b"multi-voter");
    let expiration = _env.ledger().sequence() + 1000;
    let proposal_id = client.initialize_proposal(&admin, &desc_hash, &expiration);

    client.vote(&voter1, &proposal_id, &true);
    client.vote(&voter2, &proposal_id, &false);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.total_yes_votes, 3000i128);
    assert_eq!(proposal.total_no_votes, 2000i128);
}

#[test]
#[should_panic(expected = "voter has already voted on this proposal")]
fn test_double_vote_panics() {
    let (_env, client, _admin, user, _token) = setup_test_env();

    let desc_hash = Bytes::from_slice(&_env, b"no double vote");
    let expiration = _env.ledger().sequence() + 1000;
    let proposal_id = client.initialize_proposal(&user, &desc_hash, &expiration);

    client.vote(&user, &proposal_id, &true);
    client.vote(&user, &proposal_id, &true);
}

#[test]
#[should_panic(expected = "proposal has expired")]
fn test_vote_expired_proposal() {
    let (_env, client, _admin, user, _token) = setup_test_env();

    let expiration = _env.ledger().sequence();
    let desc_hash = Bytes::from_slice(&_env, b"expired");
    let proposal_id = client.initialize_proposal(&user, &desc_hash, &expiration);

    client.vote(&user, &proposal_id, &true);
}

#[test]
fn test_get_voter_registry() {
    let (_env, client, _admin, user, _token) = setup_test_env();

    let desc_hash = Bytes::from_slice(&_env, b"registry test");
    let expiration = _env.ledger().sequence() + 1000;
    let proposal_id = client.initialize_proposal(&user, &desc_hash, &expiration);

    client.vote(&user, &proposal_id, &true);

    let registry = client.get_voter_registry(&proposal_id);
    assert!(registry.contains_key(user.clone()));
    assert_eq!(registry.get(user.clone()).unwrap(), true);
}
