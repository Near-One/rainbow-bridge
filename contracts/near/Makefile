MAKEFILE_DIR :=  $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

MANIFEST := $(MAKEFILE_DIR)/Cargo.toml

build-eth2-client:
	cargo near build non-reproducible-wasm --manifest-path $(MAKEFILE_DIR)/eth2-client/Cargo.toml --out-dir $(MAKEFILE_DIR)/res

build-prover:
	cargo near build non-reproducible-wasm --manifest-path $(MAKEFILE_DIR)/eth-prover/Cargo.toml --out-dir $(MAKEFILE_DIR)/res --no-abi

build: build-eth2-client build-prover

test:
	cargo test

check-fmt:
	cargo fmt --check

clippy-near:
	cargo clippy

check: clippy-near check-fmt
