# DNSFlow Security Model

DNSFlow operates as a low-level network utility, requiring high-privilege access on both Linux and Windows to perform per-application DNS redirection. This document outlines the security assumptions, requirements, and risks associated with running the tool.

## Privilege Requirements

DNSFlow requires **Root** (Linux) or **Administrator** (Windows) privileges for:
- **eBPF (Linux)**: Loading and attaching kernel hooks for high-fidelity process attribution (`CAP_BPF`, `CAP_NET_ADMIN`).
- **Mount Namespace (Linux)**: Using `unshare` to isolate an application's `/etc/resolv.conf`.
- **WinDivert (Windows)**: Loading the signed kernel driver for packet capture.
- **DLL Injection (Windows)**: Using `CreateRemoteThread` to inject hooks into target processes.
- **Network Interface Binding**: Binding to the default DNS port (53) on `127.0.0.53`.

## Local DNS Proxy

The application starts a local DNS proxy listening on a loopback address:
- **Default Address**: `127.0.0.53:53` (fallback: `127.0.0.1:5353`).
- **Access Control**: The proxy is accessible to any application running on the same host.
- **Risks**: A malicious application could potentially discover and use the local proxy to resolve hostnames, though DNSFlow's ancestry walking and socket-to-PID mapping will only apply routing rules to configured target processes.

## Process Attribution & Trust

DNSFlow attributes incoming DNS queries to specific processes using kernel-provided metadata:
- **Linux**: Uses `/proc/net/udp` or `/proc/net/tcp` to map local socket ports to Process IDs (PIDs).
- **Windows**: Uses the `GetExtendedUdpTable` API to map active UDP ports to PIDs.

**Security Assumption**: The mapping provided by the OS kernel is assumed to be trustworthy. A malicious process cannot easily spoof its source port or the kernel's mapping to impersonate another process for DNS routing purposes.

## Configuration Security

DNSFlow stores its configuration and logs in local files:
- **Database**: SQLite DB (`dnsflow.db`) is stored in the user's data directory.
- **Rules File**: `/tmp/dnsflow_rules.json` (Linux) or `%TEMP%\dnsflow_rules.json` (Windows).
- **Permissions**: The application sets 0644 permissions on these files to ensure they are readable but only writable by the root/Admin process.

## Redirection Bypassing

DNSFlow uses several techniques to ensure that applications cannot easily bypass its DNS redirection:
- **Mount Namespace isolation**: Forces the application to see a custom `/etc/resolv.conf`.
- **Packet Spoofing**: Captures outbound port 53 traffic directly at the network layer.
- **DoH/TRR Disabling**: Forcefully disables browser-internal DNS resolvers via CLI flags and enterprise policies.

**Risk**: An application with its own internal, hardcoded encrypted DNS resolver (like DNS-over-HTTPS) and certificate pinning could potentially bypass DNSFlow's interception unless its traffic is completely blocked or specific browser handling is applied.
