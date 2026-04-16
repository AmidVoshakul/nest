# Development Guide

## Development Workflow

### Prerequisites
- Rust 1.87+
- cargo clippy
- cargo fmt

### Build Commands
```bash
# Full build
make build

# Run checks (always run before commit)
make check

# Run all tests
make test

# Run in development mode
make run-dev
```

### Project Structure
```
nest/
├── crates/                     # Core libraries
│   ├── nest-api/              # Public traits and types
│   ├── nest-sandbox/          # Linux sandbox implementation
│   ├── nest-permissions/      # Permission engine
│   ├── nest-audit/            # Audit log system
│   ├── nest-messaging/        # Message bus
│   ├── nest-tools/            # MCP protocol client
│   ├── nest-llm/              # LLM provider integration
│   └── nest-runtime/          # Agent execution loop
├── hands/                     # Agent definitions
├── tools/                     # MCP server implementations
├── docs/                      # Documentation
└── src/                       # CLI entrypoint
```

## Adding New Features

### 1. Create Branch
```bash
git checkout -b feature/your-feature-name
```

### 2. Implement Changes
- Follow existing patterns
- Don't add dependencies without justification
- All code must compile cleanly

### 3. Run Checks
```bash
make check
make test
```

### 4. Submit Pull Request

## Coding Standards

### General
- All code must compile with no warnings
- Clippy must pass with `-D warnings`
- Use `rustfmt` for formatting
- Add documentation comments for public items

### Rust Specific
- Prefer `anyhow::Result` for internal code
- Use `nest_api::error::Result` for public API
- Prefer composition over inheritance
- Use `tokio` for async operations
- Avoid `unsafe` unless absolutely necessary

### Security
- Never add implicit permissions
- Always validate all inputs
- Audit log every meaningful action
- No secrets in code or logs

## Debugging Tips

### Enable Debug Logs
```bash
RUST_LOG=debug cargo run -- research "query"
```

### Test Individual Crates
```bash
cargo test -p nest-runtime
cargo test -p nest-sandbox
```

### Check Documentation
```bash
cargo doc --open
```

## Release Process

1. Update version in Cargo.toml
2. Run full test suite
3. Update CHANGELOG.md
4. Create git tag
5. Build release binaries
6. Publish to crates.io
