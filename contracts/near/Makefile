MAKEFILE_DIR :=  $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

MANIFEST := $(MAKEFILE_DIR)/Cargo.toml

build-eth2-client:
	cargo near build reproducible-wasm --manifest-path $(MAKEFILE_DIR)/eth2-client/Cargo.toml

build-prover:
	cargo near build reproducible-wasm --manifest-path $(MAKEFILE_DIR)/eth-prover/Cargo.toml

build: build-eth2-client build-prover

test:
	cargo test

check-fmt:
	cargo fmt --check

clippy-near:
	cargo clippy

check: clippy-near check-fmt
