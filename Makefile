default: build check test

.PHONY: clean
clean:
	cargo clean

.PHONY: build
build:
	cargo build

check: check-fmt check-clippy

.PHONY: check-fmt
check-fmt:
	cargo fmt --check

.PHONY: check-clippy
check-clippy:
	cargo clippy -- --deny warnings

.PHONY: check-test
check-test: check-clippy-test

.PHONY: check-clippy-test
check-clippy-test:
	cargo clippy --tests -- --deny warnings

.PHONY: test
test: check-test
	cargo test

.PHONY: format
format:
	cargo fmt
