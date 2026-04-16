# Security Policy

## Security Model

Nest follows a **deny-by-default** security model inspired by nanoclaw, OpenFang and ZeroClaw.

### Core Security Principles

1.  **No implicit permissions** - Every action requires explicit approval
2.  **Defense in depth** - Multiple overlapping security layers
3.  **Isolation first** - Security boundary at OS level, not application level
4.  **Audit everything** - Immutable log of all agent actions
5.  **Least privilege** - Agents get minimum permissions needed

### Security Boundaries

| Layer | Description |
|-------|-------------|
| **Kernel Level Sandbox** | Linux namespaces + pivot_root + seccomp-bpf. Agents cannot escape. |
| **Permission Engine** | Granular per-agent, per-tool permissions. Deny by default. |
| **Credential Isolation** | API keys never enter agent sandboxes. Injected at proxy level. |
| **Immutable Audit Log** | Cryptographically verified log of all actions. Cannot be tampered with. |
| **MCP Proxy** | All tool calls go through proxy with permission checks. |

### Agent Isolation

Each agent runs in:
- Separate PID namespace
- Separate network namespace
- Separate mount namespace with pivot_root
- Unprivileged user (uid 1000)
- No capabilities
- Seccomp filter with ~40 allowed syscalls
- Read-only root filesystem
- Memory and CPU limits

### Reporting a Vulnerability

If you discover a security vulnerability, please report it privately to:
security@nest-hypervisor.dev

Do NOT create public issues for security vulnerabilities.

We will respond within 24 hours and provide a fix within 72 hours for critical issues.

## Security Best Practices

1.  **Always run as unprivileged user** - Never run Nest as root
2.  **Review permissions** - Don't auto-approve everything
3.  **Keep software updated** - Security patches are released regularly
4.  **Monitor audit log** - Review agent actions periodically
5.  **Use API keys with minimal scope** - Don't give agents full admin keys

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✅ Full support |
| < 0.1   | ❌ Not supported |

## Security Features Roadmap

- [ ] Network proxy with egress filtering
- [ ] Time-bound credentials
- [ ] Agent behavior anomaly detection
- [ ] Multi-signature approval for high-risk actions
- [ ] Formal security audit
