# Aether-Red

> [!IMPORTANT]
> **ENGINEERING STATUS**: Aether-Red is **100% PRODUCTION HARDENED**. The engine has passed a deep architectural audit and surgical remediation, achieving zero-warning build integrity.

> **An Advanced Adversarial Emulation & High-Frequency Traffic Generation Framework**

Aether-Red is a sophisticated, systems-grade networking engine built in Rust. Designed for security auditing, load testing, and production-grade adversarial emulation, it provides a modular, high-performance platform for generating 30+ complex traffic patterns while maintaining a hardened security posture.

---

## 🏗️ Battle-Hardened Engineering (Remediated v2.1)

Aether-Red is engineered for sustainable, high-velocity simulations with state-level stealth:

- **⚡ Zero-Thrash Memory**: Arena/Buffer Reuse pattern + **Thread-Local Gzip Stacks**. Workers eliminate heap allocations during fuzzer execution.
- **🛡️ RST Injection & Pre-Flight Sockets**: Guaranteed SO_LINGER(0) configuration before any network activity to eliminate `TIME_WAIT` overhead.
- **🌀 Full Category C Stealth**: Fragmented TLS handshake (5ms inter-packet delay) to bypass modern DPI reassembly.
- **📊 Precise Telemetry**: High-precision hybrid timers (spin-yield logic) for sub-millisecond Poisson arrival accuracy.
- **🛡️ OOM Protection**: Streaming response consumption with 10MB body caps to prevent engine starvation from "Body Bomb" attacks.
- **🚀 Scalable Orchestration**: True **Round-Robin Tasking** across the worker swarm for linear throughput scaling.

---

## 🚥 30+ Adversarial Feature Matrix

### 🚦 Category A: Timing & Concurrency (`mod_traffic`)
- **Poisson Arrival**: Stochastic events based on high-precision exponential distribution.
- **Micro-Burst**: High-density duty-cycle oscillation (Burst/Idle).
- **Slowloris**: Targeted connection starvation at 95% timeout threshold.
- **Race Condition Trigger**: Barrier-synchronized microsecond execution.
- **Working Hours**: Local-time aware peak/idle ramping.
- **Geo-Latency**: Synthetic ACK delays and region jitter.
- **Decoy & Sniper**: Malicious payload camouflage (90/10 ratio masking).
- **Jittered Constant**: Gaussian-approximated variance on baseline RPS.
- **Heartbeat & Pulse**: Absolute periodicity for C2 simulation.

### 🧪 Category B: Protocol Abuse (`mod_fuzz`)
- **Request Smuggling**: CL.TE boundary exploitation with conflicting headers.
- **JSON Explosion**: Iterative recursive depth exhaustion (Stack-safe).
- **Compression Bombs**: Zero-allocation Gzip/Brotli bombs.
- **Oversized Headers**: 32KB+ cookie/header exhaustion.
- **Polyglot Injection**: Multi-context valid attack signatures.
- **Bad-Char Walking**: Sequential byte-mutation for exception hunting.
- **Double-Encoding**: Recursive URL encoding for WAF bypass.
- **Verb Manipulation**: Validated HEAD/PROPFIND/MOVE/SEARCH exploitation.
- **Protocol State Abuse**: Null-byte boundaries and empty-body transitions.
- **Handshake Termination**: Premature EOF/incomplete TLS preamble injection.

### 🛡️ Category C: Stealth & Identity (`mod_net/mod_auth`)
- **Fragmented TLS**: Tiny-segment (5 byte) handshake writes with mandatory delays.
- **Slow Read**: Reverse Slowloris (1 byte/500ms response consumption).
- **Protocol Downgrade**: Forced TLS 1.0/1.1 and raw HTTP/1.0 legacy paths.
- **Session Replay**: Hostile TLS ticket reuse via shared memory session stores.
- **Early Data (0-RTT)**: Replay protection testing for TLS 1.3 endpoints.
- **Cookie Stuffing**: CPU-bound session deserialization stress.
- **JA3 Cycling**: Multi-browser TLS fingerprint rotation.
- **IP Swarm**: Source-IP rotation via interface binding.
- **DNS Rebinding**: Dynamic resolver state manipulation.
- **H2 Frame Flooding**: Control frame (SETTINGS/PING) resource exhaustion.

---

## 🎭 C2 Dashboard & Visualization

- **Swarm Sampling**: Integrated sampling algorithm (1:N) allows the TUI to render 10,000+ workers at a smooth 30 FPS without CPU starvation.
- **Zombie Detection**: Heartbeat-driven liveness decay for real-time thread health tracking.
- **Boundary Sanitizer**: ANSI-stripping and Lossy UTF-8 conversion for total log safety.
- **Interactive Knobs**: On-the-fly RPS and Jitter adjustment via atomic hotkeys.

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
./target/release/aether_core
```

---

> [!IMPORTANT]
> **Legal Disclaimer**: Aether-Red is provided for authorized security auditing and research purposes only. The authors assume no liability for misuse or damage caused by this tool. Always ensure you have explicit permission before testing any target.
