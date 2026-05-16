.PHONY: all build test clean fmt clippy deploy

all: build

build:
	cargo build --release

test:
	cargo test

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy -- -D warnings

clean:
	cargo clean

wasm-size:
	@wc -c target/wasm32-unknown-unknown/release/soroban_voting.wasm

deploy-testnet:
	soroban contract deploy \
		--wasm target/wasm32-unknown-unknown/release/soroban_voting.wasm \
		--network testnet \
		--source $(SENDER)

initialize-testnet:
	soroban contract invoke \
		--id $(CONTRACT_ID) \
		--network testnet \
		--source $(SENDER) \
		-- \
		initialize \
		--admin $(ADMIN) \
		--token_address $(TOKEN) \
		--min_threshold $(THRESHOLD)
