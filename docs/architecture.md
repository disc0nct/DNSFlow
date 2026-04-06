# DNSFlow Architecture

## Introduction
DNSFlow is a per-application DNS override tool that routes DNS queries to specific upstream servers based on the originating application. While the React frontend remains agnostic to underlying operating system differences, the backend employs multiple techniques to achieve feature parity across Linux and Windows.

## Core Design Principles

### Process Attribution
The heart of DNSFlow is its ability to identify which application made a DNS query. This is achieved via **Socket-to-PID Mapping**:
- When a UDP packet arrives at the local proxy (`127.0.0.53:53`), the proxy extracts the source port.
- It then queries the kernel's network table (`/proc/net/udp` on Linux, `GetExtendedUdpTable` on Windows) to find the Process ID (PID) owning that source port.
- This mapping provides a reliable and untamperable link between a DNS query and the originating application.

### RulesEngine & Ancestry Walking
Rules are defined by the user for specific applications. However, many applications spawn child processes (e.g., browser renderer processes) that do not match the main application's process name or executable path.

To handle this, DNSFlow implements **Ancestry Walking**:
1. When a query is received, the proxy identifies the PID.
2. The `RulesEngine` checks if this PID has an explicit rule.
3. If not, it recursively examines the Parent PID (PPID) of the process.
4. This traversal continues up to **10 levels deep**.
5. If any ancestor matches a rule, the query is routed accordingly.

### Session Tracking
When an application is launched via the DNSFlow "Launch" button, its PID is recorded as a **Tracked Session**. This ensures that even if the process name changes or the binary is renamed, all queries originating from that specific launch session (including children) are routed through the selected DNS server.

## Linux Architecture
On Linux, DNSFlow uses three main interception layers:
- **Mount Namespaces**: Uses `unshare` to bind-mount a custom `/etc/resolv.conf` per-application.
- **eBPF (Kprobes)**: Attaches to `udp_sendmsg` to capture high-fidelity process attribution and connection metadata.
- **LD_PRELOAD Shim**: A shared library (`libdnsflow_shim.so`) that overrides standard libc resolution calls like `getaddrinfo`.

## Windows Architecture
On Windows, DNSFlow uses two primary techniques:
- **WinDivert (Packet-level)**: Filters outbound port 53 UDP traffic and spoofs the destination address to `127.0.0.53`, effectively routing all system DNS traffic through the local proxy.
- **DLL Injection (DNSAPI Hook)**: Injects `dnsflow_hook.dll` into target processes to hook the Windows `DnsQuery` API.

## Common Proxy Core
Regardless of the interception method, all redirected traffic flows into the shared Rust core:
- **UDP Listener**: Listens on `127.0.0.53:53` (fallback: `127.0.0.1:5353`).
- **Hickory DNS Resolver**: A highly performant and secure Rust DNS library used to forward queries to upstream servers over UDP, TCP, DNS-over-HTTPS (DoH), or DNS-over-TLS (DoT).
- **SQLite Database**: Stores user rules, DNS servers, and query logs for persistence across restarts.
