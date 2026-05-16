# Soroban Voting Engine

A `#![no_std]` Rust smart contract on Soroban (Stellar) that powers decentralized on-chain voting. Proposals are stored with cryptographic description hashes, creation is gated by a minimum token-holder threshold, and voting power is calculated directly from live token balances.

---

## Architecture

```
src/
├── lib.rs          # #![no_std] entry point, module declarations, re-exports
├── types.rs        # Proposal, ProposalState, DataKey — all #[contracttype]
├── contract.rs     # VotingContract — initialize, initialize_proposal, vote, queries
├── events.rs       # Event emission helpers using Symbol topics
└── tests.rs        # 10 integration tests covering all code paths
```

### Storage Model

| Key | Value |
|---|---|
| `DataKey::Admin` | `Address` — contract administrator |
| `DataKey::TokenAddress` | `Address` — the ecosystem token contract |
| `DataKey::MinThreshold` | `i128` — minimum token balance to create a proposal |
| `DataKey::ProposalCount` | `u32` — auto-incrementing proposal counter |
| `DataKey::Proposal(u32)` | `Proposal` — full proposal state |
| `DataKey::VoterRegistry(u32)` | `Map<Address, bool>` — per-proposal voter tracking |

All values use **instance storage** — persistent across transactions and ledger closes.

---

## Quick Start

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- `soroban` CLI 21.x (optional, for deployment)
- `make` (optional, for Makefile commands)

```bash
rustup target add wasm32-unknown-unknown
```

### Build

```bash
make build
# or: cargo build --release
```

The Wasm binary is written to `target/wasm32-unknown-unknown/release/soroban_voting.wasm`.

### Test

```bash
make test
# or: cargo test
```

10 tests covering initialization, proposal creation (sufficient and insufficient balance), yes/no vote tallying, multi-voter aggregation, double-vote rejection, expired-proposal rejection, and voter registry queries.

### Deploy (testnet)

```bash
make deploy-testnet SENDER=<your-secret-key>
# or:
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/soroban_voting.wasm \
  --network testnet \
  --source <deployer-secret>
```

### Initialize

```bash
make initialize-testnet \
  CONTRACT_ID=<contract-id> \
  SENDER=<admin-secret> \
  ADMIN=<admin-address> \
  TOKEN=<ecosystem-token-address> \
  THRESHOLD=1000000000
# or:
soroban contract invoke \
  --id <contract-id> \
  --network testnet \
  --source <admin-secret> \
  -- \
  initialize \
  --admin <admin-address> \
  --token_address <ecosystem-token-address> \
  --min_threshold 1000000000
```

---

## Contract API

### `initialize`

Gates: `admin.require_auth()` — only callable once.

```
initialize(env, admin: Address, token_address: Address, min_threshold: i128)
```

### `initialize_proposal`

Gates: `from.require_auth()` + `token.balance(from) >= min_threshold`.

Returns the new `proposal_id: u32`.

```
initialize_proposal(
    env,
    from: Address,
    description_hash: Bytes,
    expiration_ledger: u32,
) -> u32
```

| Parameter | Description |
|---|---|
| `description_hash` | Arbitrary bytes — typically a content-addressed hash (e.g. IPFS CID, SHA-256) of the full proposal text |
| `expiration_ledger` | Ledger sequence number after which voting is no longer accepted |

### `vote`

Gates: `voter.require_auth()` + proposal is `Active` + not expired + not already voted + `token.balance(voter) > 0`.

Voting power is the caller's **current token balance** at the time of `vote`. This means power can change between proposals and is recalculated fresh per call.

```
vote(env, voter: Address, proposal_id: u32, support: bool)
```

### Query Functions (read-only)

| Function | Returns |
|---|---|
| `get_proposal(proposal_id)` | `Proposal` — full state, vote tallies, creator |
| `get_proposal_count()` | `u32` — total proposals created |
| `has_voted(voter, proposal_id)` | `bool` — whether address has voted |
| `get_voter_registry(proposal_id)` | `Map<Address, bool>` — all voters on a proposal |

---

## Data Model

### `ProposalState`

```rust
pub enum ProposalState {
    Active,    // open for voting
    Passed,    // reserved for future resolution logic
    Rejected,  // reserved for future resolution logic
    Expired,   // set automatically when vote() detects expiration
}
```

### `Proposal`

```rust
pub struct Proposal {
    pub state: ProposalState,
    pub description_hash: Bytes,
    pub total_yes_votes: i128,
    pub total_no_votes: i128,
    pub expiration_ledger: u32,
    pub creator: Address,
}
```

---

## Events (Indexable)

### `proposal_created`

```js
{
    "topic": ["proposal_created"],
    "data": [proposal_id: u32, creator: Address]
}
```

### `vote_cast`

```js
{
    "topic": ["vote_cast"],
    "data": [voter: Address, proposal_id: u32, support: bool, voting_power: i128]
}
```

Events are published via `env.events().publish()` with `Symbol` topic labels, making them queryable through Soroban's event indexer (RPC `getEvents`).

---

## Security Considerations

| Concern | Mitigation |
|---|---|
| **Double-voting** | `Map<Address, bool>` per proposal — checked before every `vote()` |
| **Expired proposals** | `env.ledger().sequence()` comparison against `expiration_ledger` — enforced at vote time |
| **Unauthorized proposal creation** | `token::Client::balance()` check against `min_threshold` before allowing creation |
| **Replay / spoofed auth** | `require_auth()` on every mutating call (`initialize`, `initialize_proposal`, `vote`) |
| **Governance token manipulation** | Voting power is the live balance — if the token is a standard Stellar asset contract, flash-loan attacks are bounded by the atomicity of Soroban transactions |
| **Non-malicious expiration** | Expiration transitions state to `Expired` but does not destroy data — proposals remain queryable |

---

## Dependencies

- **soroban-sdk** `21.2.1` — Soroban contract SDK with token interface, Map, Symbol, and storage primitives
- **testutils** (dev) — `Address::generate`, `StellarAssetClient`, `Env::mock_all_auths`

---

## License

MIT
