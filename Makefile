# Nest Agent Hypervisor - Makefile

.PHONY: all build check test clean run install docs

# Default target
all: check build

# Build release binary
build:
	cargo build --release

# Development build
dev:
	cargo build

# Check compilation and run lints
check:
	cargo check
	cargo clippy -- -D warnings

# Run tests
test:
	cargo test --all-features

# Run all checks (CI mode)
ci: check test
	cargo fmt --check
	cargo doc --no-deps

# Run Nest hypervisor
run:
	cargo run -- start

# Run in development mode with debug logging
run-dev:
	RUST_LOG=nest=debug,info cargo run -- start

# Clean build artifacts
clean:
	cargo clean
	rm -rf var/* target/*

# Install binary to system
install: build
	install -m 755 target/release/nest /usr/local/bin/nest

# Generate documentation
docs:
	cargo doc --open --no-deps

# Show status of running agents
status:
	cargo run -- status

# Show audit log
log:
	cargo run -- log

# List pending permissions
permissions:
	cargo run -- permissions list

# Format code
fmt:
	cargo fmt
