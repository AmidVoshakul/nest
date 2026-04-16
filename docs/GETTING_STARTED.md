# Getting Started with Nest

## Prerequisites

### System Requirements
- Linux 6.0+ (required for modern namespace features)
- Rust 1.87+
- Root privileges (for sandbox operations)
- x86_64 architecture

### Dependencies
```bash
# Ubuntu/Debian
sudo apt install build-essential libssl-dev pkg-config

# Fedora
sudo dnf install gcc openssl-devel
```

## Installation

### 1. Clone Repository
```bash
git clone https://github.com/AmidVoshakul/nest
cd nest
```

### 2. Configure Environment
```bash
cp .env.example .env
```

Edit `.env` with your API keys:
```env
ANTHROPIC_API_KEY=your_key_here
OPENAI_API_KEY=your_key_here
OPENROUTER_API_KEY=your_key_here
```

### 3. Build Project
```bash
make build
```

### 4. Verify Installation
```bash
cargo run -- --help
```

## First Run

### Run Research Task
```bash
cargo run -- research "What are agent operating systems in 2026?"
```

This will:
1. Start the Nest runtime
2. Load the Researcher hand agent
3. Execute your research query
4. Return structured results with sources

## Basic Usage

### Start as Daemon
```bash
cargo run -- start
```

### Check Status
```bash
cargo run -- status
```

### Schedule Recurring Task
```bash
# Run every hour
cargo run -- schedule researcher "0 * * * *" "Check for new AI security news"
```

### View Scheduled Tasks
```bash
cargo run -- schedule-list
```

### Manage Permissions
```bash
# List pending requests
cargo run -- permissions list

# Approve request #5
cargo run -- permissions approve 5

# Deny request #3
cargo run -- permissions deny 3
```

## Troubleshooting

### Common Issues

1. **"Operation not permitted"**
   - You need root privileges for sandbox operations
   - Run with `sudo`

2. **"Hand not found"**
   - Make sure you're running from project root
   - Check that `./hands` directory exists

3. **LLM API errors**
   - Verify your API keys in `.env`
   - Check your internet connection

### Logs
Runtime logs are stored in `./var/log/`

## Next Steps

1. Explore the [Architecture Documentation](ARCHITECTURE.md)
2. Read the [API Reference](API.md)
3. Check out [ROADMAP.md](../ROADMAP.md) for upcoming features
4. Join our GitHub discussions
