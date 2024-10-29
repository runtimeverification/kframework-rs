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

.PHONY: test
test:
	cargo test

.PHONY: format
format:
	cargo fmt
