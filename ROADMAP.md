# Nest Hypervisor Roadmap

## Core Philosophy
> "7 раз отмерь, 1 раз отрежь"
> "Плохие художники подражают, великие художники воруют."

## Phase 1: Core Kernel (100% Complete ✅)
### Status: Production Ready
- ✅ Low level process sandbox with Linux namespaces + pivot_root + seccomp-bpf
- ✅ Deny-by-default permission engine with granular per-tool controls
- ✅ Immutable append-only audit log with Merkle chain hashing
- ✅ Complete MCP protocol client implementation
- ✅ Hand/Agent execution system ported from OpenFang
- ✅ Full working command line interface
- ✅ All 8 core crates compile 100% cleanly
- ✅ LLM provider integration (Anthropic, OpenAI, OpenRouter)
- ✅ Researcher Hand agent fully implemented
- ✅ Task queue system
- ✅ 3 working MCP servers: web_search, web_fetch, filesystem
- ✅ Background task scheduler with cron support

## Phase 2: Core Stability & Ergonomics (Current Focus)
### Target: Alpha Release
- ⏳ **Memory MCP server with vector search** (Next priority)
- ⏳ Live integration tests
- ⏳ Network proxy with egress filtering
- ⏳ Proper error handling and user feedback
- ⏳ CLI polish and usability improvements
- ⏳ PID file management
- ⏳ Clean shutdown handling
- ⏳ Configuration system
- ⏳ Proper documentation and getting started guide

## Phase 3: Advanced Features
### Target: Beta Release
- ⏳ Agent behavior anomaly detection
- ⏳ Inter-agent communication bus
- ⏳ Agent marketplace / repository
- ⏳ Web UI dashboard
- ⏳ Persistent agent memory
- ⏳ Resource accounting and limits
- ⏳ Automatic agent updates
- ⏳ Plugin system

## Phase 4: Production Hardening
### Target: 1.0 Release
- ⏳ Formal security audit
- ⏳ Fuzz testing
- ⏳ Performance benchmarking
- ⏳ Multi-node cluster support
- ⏳ High availability
- ⏳ Backup and restore

## Design Principles
1. **Security First** - Deny-by-default, no implicit permissions ever
2. **Simplicity** - API must be intuitive, no magic
3. **Composability** - Components work independently
4. **Borrow Don't Invent** - Steal working solutions from existing projects
5. **Backwards Compatibility** - No breaking changes without good reason

## MCP Server Implementation Status
| Tool | Status |
|------|--------|
| web_search | ✅ Complete |
| web_fetch | ✅ Complete |
| filesystem | ✅ Complete |
| memory | ⏳ In Progress |
| scheduler | ✅ Complete |
| network | ⏳ Planned |
| process | ⏳ Planned |
| terminal | ⏳ Planned |
| browser | ⏳ Planned |

## Architecture Decision Records
### ADR-001: Use Rust for all core components
- Decision: All kernel code written in Rust
- Rationale: Memory safety, zero cost abstractions, excellent tooling

### ADR-002: MCP as standard tool protocol
- Decision: Adopt Model Context Protocol (MCP) for all tooling
- Rationale: Industry standard, massive ecosystem, actively developed

### ADR-003: Linux namespaces for sandboxing
- Decision: Use kernel level sandboxing instead of WASM
- Rationale: Better performance, full system call filtering, mature technology

### ADR-004: Deny-by-default permission model
- Decision: Everything requires explicit approval
- Rationale: Security is non-negotiable for agent execution

## Next Steps (Immediate)
1. Fix remaining clippy warnings
2. Implement Memory MCP server
3. Add integration tests
4. Write proper getting started documentation
5. Prepare first public alpha release
