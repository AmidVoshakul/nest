# AGENTS.md - Nest Agent Hypervisor

## Project Status
✅ **100% production-ready kernel implementation**

## Architecture
8 core crates, all compile cleanly:
- `nest-api` - Core traits and types
- `nest-sandbox` - Linux namespaces + pivot_root + seccomp-bpf
- `nest-permissions` - Deny-by-default permission engine
- `nest-audit` - Immutable Merkle chain audit log
- `nest-messaging` - Inter-agent message bus
- `nest-tools` - MCP protocol client
- `nest-llm` - LLM provider integration (Anthropic, OpenAI, OpenRouter)
- `nest-runtime` - Hand/Agent execution loop

## Critical Commands
```bash
# Build project
make build

# Run checks (always run before commit)
make check

# Run all tests
make test

# Start hypervisor
make run

# Start in development mode
make run-dev

# Submit research task
cargo run -- research "topic to investigate"
```

## Development Workflow
1.  Always run `make check` before committing
2.  Security is absolute #1 priority - no implicit permissions ever
3.  Follow existing patterns, do not reinvent working solutions
4.  All new code must compile 100% cleanly before moving on
5.  Never add dependencies without justification

## Environment
- Copy `.env.example` to `.env`
- Set API keys: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `OPENROUTER_API_KEY`
- Environment variables are loaded automatically at startup

## Hands
Agents are defined in `hands/` directory with TOML manifests:
- `researcher` - Background research agent with web access
- Manifest defines permissions, tools, system prompt, resource limits

## Gotchas
- No implicit permissions - everything requires explicit approval
- API keys never enter agent sandboxes (injected at proxy level)
- Agents run in full kernel-level isolation
- All actions are logged to immutable audit log

## Standard Directories
- `crates/` - Core library code
- `hands/` - Agent definitions
- `var/` - Runtime data
- `sandbox/` - Agent filesystems
- `store/` - Persistent storage

## CI / CD
- GitHub Actions runs full test suite on every commit
- All crates must compile with no warnings
- Clippy checks are enforced

## Skills - USE ONLY THESE
When asked to use skills, only invoke:

| Category | Skills |
|----------|--------|
| Core (always) | lint-and-validate, code-reviewer, clean-code |
| Rust | rust-pro, rust-async-patterns, systems-programming-rust-project |
| Backend | backend-architect, backend-dev-guidelines |
| Security | security-audit, security-scanning-security-sast |
| DevOps | docker-expert |

**IGNORE all other skills** - they are not applicable to this project.

## OpenCode Config Location
`.opencode/opencode.json` (project-local)

## Tips
- `"Bad artists imitate, great artists steal."` - (openclaw, openfang, nanoclaw, zeroclaw, copilot, ironclaw, opencode )
