# Contributing to Nest

Thank you for your interest in contributing to Nest!

## Development Guidelines

### Code Standards

1.  **Security first** - Every line of code must have a security justification
2.  **Minimal dependencies** - Every new dependency needs explicit justification
3.  **No magic** - Code must be obvious and readable
4.  **Tests required** - Security critical code needs 100% test coverage
5.  **Follow existing patterns** - Maintain consistency with existing codebase

### Commit Messages

Follow conventional commits format:
```
type: subject

body (optional)
```

Types:
- `feat:` New feature
- `fix:` Bug fix
- `refactor:` Code change that neither fixes a bug nor adds a feature
- `docs:` Documentation changes
- `test:` Adding or modifying tests
- `chore:` Build process, tooling, etc.

### Pull Request Process

1.  Fork the repository
2.  Create a feature branch
3.  Make your changes
4.  Run full test suite: `make ci`
5.  Submit pull request with clear description

### Security Critical Changes

For changes affecting security boundaries:
- Add comprehensive tests
- Include security justification in PR description
- Expect thorough review process

## Building

```bash
# Development build
make dev

# Release build
make build

# Run all checks
make ci
```

## Running

```bash
# Copy example config
cp .env.example .env
# Edit .env with your API keys

# Run in development mode
make run-dev
```

## Project Structure

```
nest/
├── crates/
│   ├── nest-api/          # Core public traits and types
│   ├── nest-sandbox/      # Low level process isolation
│   ├── nest-permissions/  # Permission engine
│   ├── nest-audit/        # Immutable audit log
│   ├── nest-messaging/    # Inter-agent message bus
│   ├── nest-tools/        # MCP protocol client
│   ├── nest-llm/          # LLM provider integration
│   └── nest-runtime/      # Agent execution loop
├── hands/                 # Hand agent definitions
├── docs/                  # Documentation
├── tests/                 # Integration tests
└── src/                   # CLI entrypoint
```

## Code Review

All changes require review before merging. Focus areas:
1.  Security implications
2.  Performance impact
3.  Backwards compatibility
4.  Test coverage

We reserve the right to reject changes that don't align with project goals.

## Community

- Join our Discord for discussions
- Check GitHub issues for good first issues
- Read ARCHITECTURE.md for design decisions

## License

By contributing to Nest, you agree that your contributions will be licensed under the MIT license.
