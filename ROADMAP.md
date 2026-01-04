# Aether-Red: Comprehensive Feature Roadmap

This document outlines the high-level technical specifications and adversarial strategies categorized by module.

## Category A: `mod_traffic` (Timing & Concurrency Strategies)
Tests for: **DDoS Defense, Rate Limiting Algorithms, Auto-Scaling Triggers.**

- **The Poisson Arrival**: Simulate organic user traffic where request intervals follow a Poisson distribution. Tests if the target's autoscaler reacts to "natural" crowd surges versus linear bot ramps.
- **The Micro-Burst**: Send 5,000 requests in 100ms, then sleep for 5 seconds. Repeat. Tests if WAF "Bucket Token" algorithms allow packets through during refill intervals.
- **The Slowloris (Connection Starvation)**: Open thousands of connections but write headers 1 byte at a time (e.g., write_interval: 10s). Tests keep-alive timeouts and thread pool limits.
- **The Heartbeat (C2 Simulation)**: Beaconing traffic pattern (exact same request every exactly 30.0s). Tests if Threat Intelligence systems flag mechanical periodicity.
- **The "Working Hours" Simulation**: Ramp traffic up over 4 hours (Morning), dip for 1 hour (Lunch), peak again, then drop off. Tests long-duration memory leaks or log rotation issues.
- **Jittered Constant**: Target 1000 RPS with ±20% randomized jitter. Standard baseline testing.
- **Race Condition Trigger**: Sync multiple workers to fire a request at the exact same microsecond (using a Barrier primitive). Tests data integrity in database transactions (e.g., "Double Spend").
- **Connection Churn Storm**: Establish and immediately tear down TCP connections without sending data (SYN Flood behavior). Tests Kernel network stack/firewall conntrack tables.
- **The Decoy & Sniper**: 90% benign "noise" vs 10% malicious traffic. Tests SIEM log aggregation rules.
- **Geo-Latency Emulation**: Deliberately delay request ACKs to simulate 3G networks or different continents. Tests application-layer timeouts.

## Category B: `mod_fuzz` (Payload & Protocol Abuse)
Tests for: **WAF Rules, Parser Logic, Memory Safety.**

- **HTTP Request Smuggling (CL.TE)**: Send headers with conflicting Content-Length and Transfer-Encoding. Tests inconsistent boundary interpretation between proxies and backends.
- **Recursive JSON Explosion**: Send JSON objects nested 5,000 layers deep. Tests stack overflow vulnerabilities in JSON parsers.
- **Oversized Headers**: Inject Cookie or User-Agent headers larger than 8KB. Tests buffer handling in the HTTP Daemon (Apache/Nginx).
- **Gzip/Brotli Bombs**: Send payloads with `Content-Encoding: gzip` containing highly compressed zeroes that expand to gigabytes. Tests resource exhaustion.
- **Polyglot Injection**: Inject strings valid in multiple contexts (e.g., SQLi + XSS in one payload). Tests input sanitation libraries.
- **Bad-Char Walking**: Sequentially byte-swap values (0x00 to 0xFF) to identify characters causing unhandled exceptions.
- **Empty/Null Body**: Send POST requests with conflicting Content-Length vs body content. Tests parser state machines.
- **Double-Encoded URL**: Send path traversals using double-encoding (e.g., `%252e%252e%252f`). Tests recursive decoding logic in WAFs.
- **Method Switcheroo**: Use non-standard verbs (HEAD, OPTIONS, PROPFIND) for attacks. Tests if WAFs only monitor GET/POST.
- **Premature EOF**: Close TCP sockets halfway through large POST bodies. Tests backend error handling and zombie process cleanup.

## Category C: `mod_net` & `mod_auth` (Infrastructure & Identity)
Tests for: **Fingerprinting, TLS Configurations, Logical Flaws.**

- **JA3 Cycling**: Rotate TLS ClientHello parameters every 10 requests (Chrome -> Firefox -> Safari -> Curl). Tests "Bot Score" heuristics.
- **Session Ticket Replay**: Aggressively reuse TLS Session Resumption tickets. Tests session cache and replay protection.
- **Protocol Downgrade**: Force connections using HTTP/1.0 or TLS 1.1. Tests legacy code paths with weaker security.
- **Early Data (0-RTT)**: Send data in the first TLS packet. Tests replay protection on the edge.
- **Cookie Stuffing**: Send requests with 100+ random cookies. Tests if the application wastes CPU cycles deserializing session states.
- **IP Rotation (The Swarm)**: Bind to different local interfaces or rotate SOCKS5 proxies per request. Tests IP-based rate limiting.
- **H2 Frame Flooding**: flood HTTP/2 PING, SETTINGS, or RST_STREAM frames. Tests HTTP/2 implementation stability.
- **DNS Rebinding**: Emulate DNS switching mid-attack to test cross-origin or proxy trust.
- **Fragmented TLS**: Split TLS Handshake packets into tiny TCP segments. Tests DPI reassembly buffers.
- **Slow Read (Reverse Slowloris)**: Read the response incredibly slowly (1 byte/sec) to fill server output buffers/memory.
