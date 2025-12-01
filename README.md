# Project ÆTHER

> **A Modular, Security-Focused Mass Communication Engine**

Project ÆTHER is a sophisticated Rust-based communication platform designed with modularity, security, and resilience at its core. Built on an actor-based concurrency model, it features pluggable traffic shaping, content generation, and authentication modules, all orchestrated through a secure terminal interface.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

---

## Table of Contents

- [Features](#features)
- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Native Installation](#native-installation)
  - [Docker Installation](#docker-installation)
- [Configuration](#configuration)
- [Usage](#usage)
  - [Running the Application](#running-the-application)
  - [Using the Terminal UI](#using-the-terminal-ui)
  - [Hot-Reloading Configuration](#hot-reloading-configuration)
- [Testing](#testing)
- [Project Structure](#project-structure)
- [Core Components](#core-components)
- [Modules](#modules)
- [Security Considerations](#security-considerations)
- [Development](#development)
- [Troubleshooting](#troubleshooting)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## Features

### Core Capabilities

- **🎭 Modular Architecture**: Trait-based plugin system for traffic shaping, content generation, and authentication
- **⚡ Actor-Based Concurrency**: Tokio-powered async runtime with zero-copy message passing
- **💾 Persistent State Management**: SQLite ledger with WAL mode, deduplication, and 24-hour rolling limits
- **🌐 Advanced Networking**: SOCKS5 proxy support with real-time health checking, DNS over HTTPS
- **🖥️ Interactive TUI**: Terminal-based dashboard built with Ratatui
- **🔄 Hot-Reloadable Config**: File-watching configuration system with zero downtime
- **🔒 Security-First Design**: Argon2id password hashing, memory zeroization, anti-debugging foundations
- **🐳 Docker Ready**: Multi-stage containerization with production-grade setup

### Pluggable Modules

- **Traffic Shaping**: SmoothFlow (consistent delays) and StealthJitter (adaptive backoff)
- **Content Generation**: Spintax resolution, ZWSP injection, UUID comment generation
- **Account Management**: Headless browser resurrection (stub for CAPTCHA solving)

---

## Architecture

Project ÆTHER follows a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────┐
│                    Terminal UI (TUI)                    │
│                   [aether_tui crate]                    │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                    Engine Core                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Supervisor │  │    Workers   │  │    Ledger    │  │
│  │   (Actor)    │  │   (Pool)     │  │  (SQLite)    │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│         [aether_core crate]                             │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                  Network Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Proxy Manager│  │  DoH Client  │  │ TLS Spoofer  │  │
│  │  (Sidecar)   │  │              │  │   (Stub)     │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│         [aether_net crate]                              │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│              Pluggable Modules                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │mod_traffic   │  │ mod_content  │  │  mod_auth    │  │
│  │(Strategies)  │  │ (Generators) │  │(Resurrection)│  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Separation of Concerns**: Each crate has a single, well-defined responsibility
2. **Trait-Based Extensibility**: New strategies can be added without modifying core code
3. **Fail-Safe Defaults**: System degrades gracefully when components fail
4. **Zero-Trust Networking**: All external communication routed through validated proxies
5. **Audit Trail**: Every action logged to persistent storage with timestamps

---

## Prerequisites

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+, Debian 11+, or equivalent)
- **CPU**: 2+ cores recommended
- **RAM**: 2GB minimum, 4GB recommended
- **Disk**: 500MB for application + space for logs and database

### Software Dependencies

#### For Native Installation

1. **Rust Toolchain** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Build Essentials**
   ```bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install -y build-essential pkg-config libssl-dev
   
   # Fedora/RHEL
   sudo dnf install -y gcc gcc-c++ openssl-devel
   ```

3. **SQLite** (usually pre-installed)
   ```bash
   # Ubuntu/Debian
   sudo apt-get install -y sqlite3 libsqlite3-dev
   ```

#### For Docker Installation

1. **Docker Engine** (20.10+)
   ```bash
   curl -fsSL https://get.docker.com -o get-docker.sh
   sudo sh get-docker.sh
   sudo usermod -aG docker $USER
   ```

2. **Docker Compose** (2.0+)
   ```bash
   sudo apt-get install -y docker-compose-plugin
   ```

### Optional Dependencies

- **Tor** (for SOCKS5 proxy): `sudo apt-get install tor`
- **Chromium** (for headless browser features): `sudo apt-get install chromium-browser`

---

## Installation

### Native Installation

1. **Clone the Repository**
   ```bash
   git clone https://github.com/yourusername/cryptography_project.git
   cd cryptography_project
   ```

2. **Build the Project**
   ```bash
   cargo build --release
   ```
   
   This will compile all crates and modules. The binary will be located at:
   ```
   target/release/aether_core
   ```

3. **Create Required Directories**
   ```bash
   mkdir -p data logs config
   ```

4. **Initialize Configuration**
   ```bash
   cp config/config.json.example config/config.json
   # Edit config/config.json with your settings
   ```

5. **Verify Installation**
   ```bash
   ./target/release/aether_core --version
   ```

### Docker Installation

1. **Clone the Repository**
   ```bash
   git clone https://github.com/yourusername/cryptography_project.git
   cd cryptography_project
   ```

2. **Build the Container**
   ```bash
   docker-compose build
   ```

3. **Start the Application**
   ```bash
   docker-compose up -d
   ```

4. **View Logs**
   ```bash
   docker-compose logs -f aether
   ```

5. **Stop the Application**
   ```bash
   docker-compose down
   ```

---

## Configuration

### Configuration File

The application reads from `config/config.json`. Here's the default configuration:

```json
{
  "max_workers": 5,
  "max_daily_per_sender": 500,
  "proxy_check_interval_secs": 30,
  "traffic_strategy": "smooth"
}
```

### Configuration Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `max_workers` | integer | 5 | Number of concurrent worker actors |
| `max_daily_per_sender` | integer | 500 | Maximum sends per sender per 24 hours |
| `proxy_check_interval_secs` | integer | 30 | Proxy health check frequency |
| `traffic_strategy` | string | "smooth" | Traffic shaping strategy: "smooth" or "stealth" |

### Environment Variables

For Docker deployments, you can override settings via environment variables:

```bash
# docker-compose.yml
environment:
  - AETHER_MAX_WORKERS=10
  - AETHER_STRATEGY=stealth
  - RUST_LOG=info
```

### Proxy Configuration

Edit `crates/aether_net/src/lib.rs` to configure your proxy list:

```rust
let proxies = vec![
    "socks5://127.0.0.1:9050".to_string(), // Tor
    "socks5://your-proxy:1080".to_string(),
];
```

> **Note**: For production use, load proxies from a configuration file or database.

---

## Usage

### Running the Application

#### Native Execution

```bash
# Run in foreground
./target/release/aether_core

# Run in background
nohup ./target/release/aether_core > /dev/null 2>&1 &
```

#### Docker Execution

```bash
# Start in detached mode
docker-compose up -d

# Start with logs visible
docker-compose up

# Restart after config changes
docker-compose restart
```

### Using the Terminal UI

The TUI provides a real-time dashboard of system status:

```
┌─────────────────────────────────────────────────────────┐
│     Project ÆTHER - Secure Terminal Engine             │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│ Dashboard                                               │
│                                                         │
│ System Status: ACTIVE                                   │
│ Workers: 5                                              │
│ Proxies: Checking...                                    │
│                                                         │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│ Press 'q' to Quit                                       │
└─────────────────────────────────────────────────────────┘
```

**Keyboard Controls**:
- `q`: Quit the application gracefully
- (Future: `r` for reload, `s` for stats, `p` for pause)

### Hot-Reloading Configuration

The system watches `config/config.json` for changes:

1. Edit the configuration file:
   ```bash
   nano config/config.json
   ```

2. Save the file (Ctrl+O, Enter, Ctrl+X)

3. The system automatically detects changes and reloads:
   ```
   [INFO] Config file changed, reloading...
   [INFO] Config reloaded successfully
   ```

No restart required!

---

## Testing

### Unit Tests

Run the test suite for all crates:

```bash
cargo test --all
```

Run tests for a specific crate:

```bash
cargo test -p aether_core
cargo test -p aether_net
cargo test -p mod_traffic
```

### Integration Tests

Test the full system build:

```bash
cargo build --release
```

Verify no errors in the output. Warnings about unused fields are expected (reserved for future features).

### Manual Testing

1. **Test Proxy Health Checker**:
   - Start Tor: `sudo systemctl start tor`
   - Run the application and check logs for proxy status
   - Expected: `Proxy Health Check Sidecar started`

2. **Test Configuration Hot-Reload**:
   - Start the application
   - Modify `config/config.json`
   - Check logs for reload confirmation

3. **Test Ledger Persistence**:
   - Run the application
   - Check that `data/aether.db` is created
   - Verify schema: `sqlite3 data/aether.db ".schema"`

### Docker Testing

```bash
# Build and test container
docker-compose build
docker-compose up

# Check container health
docker-compose ps

# Inspect logs
docker-compose logs aether | grep ERROR
```

---

## Project Structure

```
cryptography_project/
├── Cargo.toml                 # Workspace definition
├── Dockerfile                 # Production container
├── docker-compose.yml         # Orchestration config
├── README.md                  # This file
├── .gitignore                # Git exclusions
│
├── config/
│   └── config.json           # Runtime configuration
│
├── crates/
│   ├── aether_core/          # Main application
│   │   ├── src/
│   │   │   ├── main.rs       # Entry point
│   │   │   ├── lib.rs        # Core initialization
│   │   │   ├── engine.rs     # Actor supervisor
│   │   │   ├── worker.rs     # Task executors
│   │   │   ├── ledger.rs     # Persistent storage
│   │   │   ├── config.rs     # Hot-reload config
│   │   │   └── crypto.rs     # Cryptographic utilities
│   │   └── Cargo.toml
│   │
│   ├── aether_net/           # Networking layer
│   │   ├── src/
│   │   │   ├── lib.rs        # Network initialization
│   │   │   ├── proxy.rs      # Proxy manager + health check
│   │   │   ├── doh.rs        # DNS over HTTPS
│   │   │   └── tls.rs        # TLS fingerprinting (stub)
│   │   └── Cargo.toml
│   │
│   ├── aether_tui/           # Terminal UI
│   │   ├── src/
│   │   │   └── lib.rs        # Ratatui dashboard
│   │   └── Cargo.toml
│   │
│   └── aether_traits/        # Trait definitions
│       ├── src/
│       │   └── lib.rs        # TrafficStrategy, ContentGenerator, etc.
│       └── Cargo.toml
│
└── modules/                  # Pluggable implementations
    ├── mod_traffic/          # Traffic shaping strategies
    │   ├── src/
    │   │   └── lib.rs        # SmoothFlow, StealthJitter
    │   └── Cargo.toml
    │
    ├── mod_content/          # Content generation
    │   ├── src/
    │   │   └── lib.rs        # SpintaxGenerator
    │   └── Cargo.toml
    │
    └── mod_auth/             # Authentication/resurrection
        ├── src/
        │   └── lib.rs        # HeadlessResurrector (stub)
        └── Cargo.toml
```

---

## Core Components

### Engine Core (`aether_core`)

**Responsibility**: Orchestrates the entire system

- **EngineCore**: Supervisor actor that manages worker pool
- **Worker**: Task execution units with pluggable strategies
- **LedgerManager**: SQLite-based persistence with deduplication
- **ConfigWatcher**: Hot-reloadable configuration system
- **CryptoManager**: Argon2id password hashing

**Key Files**:
- [engine.rs](crates/aether_core/src/engine.rs) - Actor supervision
- [worker.rs](crates/aether_core/src/worker.rs) - Task execution
- [ledger.rs](crates/aether_core/src/ledger.rs) - Persistent storage

### Network Layer (`aether_net`)

**Responsibility**: Manages all external communication

- **ProxyManager**: SOCKS5 proxy pool with health checking
- **DohClient**: DNS over HTTPS for leak prevention
- **TlsFingerprinter**: JA3 spoofing (stub)

**Features**:
- Real-time proxy validation every 30 seconds
- Automatic dead proxy detection
- Latency measurement for optimal routing

**Key Files**:
- [proxy.rs](crates/aether_net/src/proxy.rs) - Proxy management
- [doh.rs](crates/aether_net/src/doh.rs) - DNS resolution

### Terminal UI (`aether_tui`)

**Responsibility**: User interface and system monitoring

- Built with `ratatui` for cross-platform terminal rendering
- Real-time status updates
- Keyboard-driven navigation

**Key Files**:
- [lib.rs](crates/aether_tui/src/lib.rs) - TUI implementation

---

## Modules

### Traffic Shaping (`mod_traffic`)

Implements adaptive delay strategies:

**SmoothFlow**:
- Consistent 200-400ms delays
- Low variance for normal operation
- Ideal for high-volume, low-risk scenarios

**StealthJitter**:
- Adaptive backoff based on error rate
- Exponential delay increase on failures
- Mimics human behavior patterns

**Usage**:
```rust
let strategy: Arc<dyn TrafficStrategy> = Arc::new(StealthJitter);
let delay = strategy.calculate_delay(&metrics).await;
```

### Content Generation (`mod_content`)

Implements message obfuscation:

**SpintaxGenerator**:
1. **Spintax Resolution**: `{Hello|Hi|Greetings}` → random choice
2. **ZWSP Injection**: Invisible Unicode characters for hash variation
3. **UUID Comments**: Unique HTML comments per message

**Example**:
```rust
let gen = SpintaxGenerator;
let (subject, body) = gen.generate_content("{Special|Exclusive} Offer").await?;
// Output: "Special Offer" or "Exclusive Offer" with obfuscation
```

### Account Management (`mod_auth`)

**HeadlessResurrector** (Stub):
- Designed for CAPTCHA solving via headless browser
- Currently returns mock results
- Future: Full Chromium integration

---

## Security Considerations

### Current Implementations

1. **Password Security**:
   - Argon2id with memory-hard parameters
   - Zeroization of sensitive data in memory
   - No plaintext password storage

2. **Network Security**:
   - All traffic routable through SOCKS5 proxies
   - DNS over HTTPS to prevent leaks
   - TLS fingerprinting foundation

3. **Data Security**:
   - SQLite with WAL mode for crash recovery
   - Future: SQLCipher encryption support
   - Audit trail for all operations

### Best Practices

1. **Proxy Usage**:
   - Use dedicated, trusted proxy servers
   - Rotate proxies regularly
   - Monitor health check logs for failures

2. **Configuration**:
   - Store `config.json` with restricted permissions: `chmod 600 config/config.json`
   - Never commit sensitive data to version control
   - Use environment variables for secrets in production

3. **Operational Security**:
   - Run in isolated Docker containers
   - Limit network capabilities to minimum required
   - Monitor logs for anomalies: `tail -f logs/aether.log`

4. **Rate Limiting**:
   - Respect the `max_daily_per_sender` limit
   - Use "stealth" strategy for sensitive operations
   - Monitor ledger for duplicate prevention

### Disclaimer

> **⚠️ IMPORTANT**: This software is provided for educational and research purposes. Users are solely responsible for ensuring compliance with all applicable laws and regulations. The authors assume no liability for misuse.

---

## Development

### Building from Source

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# Build specific crate
cargo build -p aether_core
```

### Running Tests

```bash
# All tests
cargo test --all

# With output
cargo test --all -- --nocapture

# Specific test
cargo test -p aether_core test_name
```

### Code Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

### Linting

```bash
# Run Clippy
cargo clippy --all -- -D warnings

# Fix auto-fixable issues
cargo clippy --all --fix
```

### Adding a New Module

1. Create module directory:
   ```bash
   mkdir -p modules/mod_newfeature/src
   ```

2. Add to workspace in `Cargo.toml`:
   ```toml
   members = [
       # ...
       "modules/mod_newfeature",
   ]
   ```

3. Implement trait from `aether_traits`

4. Register in `aether_core`

---

## Troubleshooting

### Common Issues

#### Build Errors

**Problem**: `error: linker 'cc' not found`

**Solution**:
```bash
sudo apt-get install build-essential
```

**Problem**: `error: failed to run custom build command for openssl-sys`

**Solution**:
```bash
sudo apt-get install pkg-config libssl-dev
```

#### Runtime Errors

**Problem**: `Failed to connect to database`

**Solution**:
```bash
# Ensure data directory exists
mkdir -p data

# Check permissions
chmod 755 data
```

**Problem**: `Proxy Health Check Sidecar failed`

**Solution**:
```bash
# Verify proxy is running
curl --socks5 127.0.0.1:9050 https://check.torproject.org

# Check proxy configuration in aether_net/src/lib.rs
```

#### Docker Issues

**Problem**: `permission denied while trying to connect to Docker daemon`

**Solution**:
```bash
sudo usermod -aG docker $USER
newgrp docker
```

**Problem**: `Error response from daemon: Conflict. The container name is already in use`

**Solution**:
```bash
docker-compose down
docker-compose up
```

### Logs

Check application logs for detailed error messages:

```bash
# Native
tail -f logs/aether.log

# Docker
docker-compose logs -f aether
```

### Getting Help

1. Check the [Issues](https://github.com/yourusername/cryptography_project/issues) page
2. Review logs for error messages
3. Verify prerequisites are installed
4. Ensure configuration is valid JSON

---

## Roadmap

### Phase 1: Foundation (✅ Complete)
- [x] Modular workspace architecture
- [x] Actor-based engine with worker pool
- [x] Persistent ledger with deduplication
- [x] Proxy health checking
- [x] Terminal UI
- [x] Hot-reloadable configuration

### Phase 2: Network Enhancement (🚧 In Progress)
- [ ] Full SMTP client implementation
- [ ] TLS fingerprinting (JA3 spoofing)
- [ ] Advanced proxy rotation algorithms
- [ ] Network failure recovery

### Phase 3: Content & Intelligence (📋 Planned)
- [ ] LLM integration for content generation
- [ ] Advanced spintax parser
- [ ] Template management system
- [ ] A/B testing framework

### Phase 4: "God Mode" Features (🔮 Future)
- [ ] Headless browser with CAPTCHA solving
- [ ] Chameleon traffic shaping (ML-based)
- [ ] Steganographic exfiltration
- [ ] Hardware-backed key storage (Intel SGX)
- [ ] Account resurrection system

### Phase 5: Production Hardening (🔮 Future)
- [ ] Comprehensive test coverage (>80%)
- [ ] Performance benchmarking
- [ ] Security audit
- [ ] API documentation
- [ ] Deployment automation

---

## Contributing

We welcome contributions! Please follow these guidelines:

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Pass all Clippy lints (`cargo clippy`)
- Write tests for new features
- Document public APIs

### Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new traffic shaping strategy
fix: resolve proxy health check timeout
docs: update installation instructions
test: add ledger deduplication tests
```

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- **Tokio**: Async runtime foundation
- **Ratatui**: Terminal UI framework
- **SQLx**: Type-safe SQL toolkit
- **Argon2**: Password hashing algorithm
- **Rust Community**: For excellent tooling and support

---

## Contact

- **Project Lead**: [Your Name](mailto:your.email@example.com)
- **Repository**: [GitHub](https://github.com/yourusername/cryptography_project)
- **Issues**: [Issue Tracker](https://github.com/yourusername/cryptography_project/issues)

---

**Built with ❤️ and Rust**

*Last Updated: 2025-12-02*
