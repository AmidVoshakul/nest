# Nest Architecture

## C4 Model Diagrams

### Level 1: System Context

```mermaid
C4Context
    title Nest Hypervisor - System Context

    Person(user, "User", "Person using Nest")
    System_Ext(llm, "LLM Provider", "Anthropic, OpenAI, OpenRouter")
    System_Ext(tools, "MCP Tools", "External MCP tool servers")
    System(nest, "Nest Hypervisor", "Secure agent execution environment")
    
    Rel(user, nest, "Submits tasks")
    Rel(nest, llm, "Uses for reasoning")
    Rel(nest, tools, "Uses for actions")
```

### Level 2: Container Diagram

```mermaid
C4Container
    title Nest Hypervisor - Container Diagram

    Person(user, "User")
    
    System_Boundary(nest, "Nest Hypervisor") {
        Container(runtime, "Agent Runtime", "Rust", "Main execution loop and scheduler")
        Container(perm, "Permission Engine", "Rust", "Deny-by-default permission checks")
        Container(audit, "Audit Log", "Rust", "Immutable Merkle chain log")
        Container(bus, "Message Bus", "Rust", "Inter-agent communication")
        Container(mcp, "MCP Proxy", "Rust", "Tool execution proxy")
        Container(sandbox, "Sandbox Manager", "Rust", "Linux namespace manager")
        
        ContainerDb(db, "State Store", "SQLite", "Persistent state storage")
    }
    
    System_Ext(llm, "LLM Provider")
    System_Ext(tools, "MCP Tools")

    Rel(user, runtime, "CLI / HTTP")
    Rel(runtime, perm, "Checks permissions")
    Rel(runtime, bus, "Routes messages")
    Rel(runtime, mcp, "Executes tools")
    Rel(runtime, sandbox, "Spawns agents")
    Rel(runtime, audit, "Logs all actions")
    Rel(mcp, tools, "MCP protocol")
    Rel(runtime, llm, "LLM API")
    Rel(runtime, db, "Persists state")
```

### Level 3: Component Diagram

```mermaid
C4Component
    title Nest Runtime - Component Diagram

    Container_Boundary(runtime, "Agent Runtime") {
        Component(scheduler, "Task Scheduler", "Cron parser, job queue")
        Component(hand_manager, "Hand Manager", "Hand lifecycle management")
        Component(think_cycle, "Think Cycle Executor", "LLM + Tools execution loop")
        Component(perm_router, "Permission Router", "Routes approval requests")
    }
    
    Component(perm, "Permission Engine")
    Component(audit, "Audit Log")
    Component(mcp, "MCP Proxy")
    Component(sandbox, "Sandbox Manager")
    
    Rel(scheduler, hand_manager, "Dispatches scheduled tasks")
    Rel(hand_manager, think_cycle, "Runs think cycles")
    Rel(think_cycle, mcp, "Calls tools")
    Rel(think_cycle, perm, "Checks permissions")
    Rel(think_cycle, audit, "Logs decisions")
    Rel(hand_manager, sandbox, "Spawns agent processes")
```

## Core Data Flow

```mermaid
sequenceDiagram
    participant U as User
    participant R as Runtime
    participant H as Hand Agent
    participant P as Permission Engine
    participant M as MCP Proxy
    participant T as MCP Tool
    participant A as Audit Log

    U->>R: Submit task
    R->>H: Add to task queue
    R->>H: Run think cycle
    H->>R: Request tool call
    R->>P: Check permission
    alt Permission granted
        P->>R: Allow
        R->>M: Execute tool
        M->>T: MCP Call
        T->>M: Result
        M->>H: Tool result
    else Permission required
        P->>R: Pending
        R->>U: Request approval
        U->>R: Approve
        R->>M: Execute tool
    end
    H->>R: Task complete
    R->>A: Log action
    R->>U: Return result
```

## Design Principles

1. **Security First**: Every operation goes through permission check
2. **Isolation**: No shared memory between agents
3. **Auditability**: Every action is permanently logged
4. **Simplicity**: Minimal API surface, no magic
5. **Composability**: Components work independently

## Security Guarantees

- ✅ Agents cannot escape sandboxes
- ✅ No implicit permissions for any operation
- ✅ All network access is filtered
- ✅ Audit log cannot be modified or deleted
- ✅ Resource limits are enforced at kernel level
