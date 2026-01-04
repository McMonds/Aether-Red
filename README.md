# Aether-Red

> [!CAUTION]
> **DEVELOPMENT STATUS**: Aether-Red is currently in the **TEST PHASE**. The engine is undergoing rigorous adversarial validation and benchmark calibration.

> **An Advanced Adversarial Emulation & High-Frequency Traffic Generation Framework**

Aether-Red is a sophisticated, systems-grade networking engine built in Rust. Designed for security auditing, load testing, and production-grade adversarial emulation, it provides a modular, high-performance platform for generating 30+ complex traffic patterns while maintaining a hardened security posture.

---

## 🏗️ Battle-Hardened Engineering (Directives 1-8)

Aether-Red is engineered for 24-hour sustained high-velocity simulations:

- **⚡ Zero-Thrash Memory**: Implements an **Arena/Buffer Reuse** pattern. Workers allocate persistent 1MB `BytesMut` buffers at startup, eliminating 500k+ heap allocations/sec.
- **🛡️ RST Injection (SO_LINGER)**: Guaranteed **Pre-Flight Socket Configuration**. Sets `linger(0)` before connection establishment to eliminate the `TIME_WAIT` race condition and prevent port exhaustion.
- **🌀 Tri-State TLS Architecture**: Dynamic rotation between **Native (Rustls)**, **Legacy (OpenSSL)**, and **Chrome (BoringSSL stub)** providers to vary JA3 fingerprints.
- **📡 ALPN Synchronization**: Automatic protocol-aware downgrading. Enforces consistency between TLS negotiation (`h2`/`http/1.1`) and fuzzer payloads to prevent `PROTOCOL_ERROR` triggers.
- **🚀 Distributed Swarm**: Custom Tokio orchestration with CPU-pinned worker threads and `RLIMIT_NOFILE` elevation to 65,535.
- **📊 Fire-and-Forget Telemetry**: Non-blocking `flume` ring buffer for microsecond-resolution telemetry prevents logging backpressure on the attack loop.

---

## 🚥 30+ Adversarial Feature Matrix

### 🚦 Category A: Timing & Concurrency (`mod_traffic`)
- **Poisson Arrival**: Stochastic events based on exponential distribution.
- **Micro-Burst**: High-frequency duty-cycle oscillation for buffer stress.
- **Slowloris**: Jittered connection occupancy at 95% timeout threshold.
- **Race Condition Trigger**: Barrier-synchronized swarm execution for microsecond concurrency.
- **Working Hours**: Local-time aware RPS ramping (9-5 peaks / night-time idle).
- **Geo-Latency**: Synthetic propagation delay and jitter simulation.
- **Decoy & Sniper**: Camouflaged 90/10 ratio traffic masking (background noise).
- **Jittered Constant**: Gaussian-approximated variance on baseline RPS.
- **Heartbeat & Pulse**: Absolute periodicity for C2 simulation.

### 🧪 Category B: Protocol Abuse (`mod_fuzz`)
- **Request Smuggling**: CL.TE boundary exploitation with conflicting headers.
- **JSON Explosion**: Iterative 1000-level nesting (stack-safe state machine).
- **Compression Bombs**: Recursive Gzip/Brotli deflation stress.
- **Oversized Headers**: 8KB+ cookie/header exhaustion.
- **Polyglot Injection**: Multi-context SQLi/XSS/SSTI payloads.
- **Bad-Char Walking**: Iterative byte-swap mutation for exception hunting.
- **Double-Encoding**: Recursive URL encoding for WAF bypass.
- **Verb Manipulation**: PROPFIND/MOVE/LOCK/UNLOCK/SEARCH/PURGE rotation.
- **Protocol State Abuse**: Null-byte injection and malformed state transitions.
- **Handshake Termination**: Premature EOF/incomplete TLS preamble injection.

### 🛡️ Category C: Stealth & Identity (`mod_net/mod_auth`)
- **JA3 Cycling**: Multi-provider TLS fingerprint rotation.
- **IP Swarm**: Source-IP rotation via `TcpSocket::bind()` interface selection.
- **DNS Rebinding**: TTL-agnostic resolver state manipulation.
- **H2 Frame Flooding**: Control frame (SETTINGS/PING) resource exhaustion.
- **OOM Protection**: Sharded `DashMap` storage with atomic cleanup ticks.

---

## 📦 Workspace Architecture

- **`aether_core`**: The central orchestrator and worker hive.
- **`aether_net`**: Hardened networking layer (IP Swarm, DoH, TLS).
- **`aether_traits`**: Shared interfaces for fuzzer and traffic extensions.
- **`aether_tui`**: Real-time Terminal Dashboard (`ratatui`).
- **`modules/`**: Pluggable adversarial intelligence modules.

---

## 🚦 Quick Start

### Build
```bash
cargo build --release
```

### Execution
```bash
# Ensure FD limits are elevated or run with sufficient privileges
./target/release/aether_core
```

---

> [!IMPORTANT]
> **Legal Disclaimer**: Aether-Red is provided for authorized security auditing and research purposes only. The authors assume no liability for misuse or damage caused by this tool. Always ensure you have explicit permission before testing any target.
