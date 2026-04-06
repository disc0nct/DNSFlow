# DNSFlow Network Interception Techniques

DNSFlow uses multiple techniques to intercept and route DNS queries on a per-application basis. Because operating systems do not natively support per-process DNS configuration, DNSFlow must intercept network traffic or API calls to achieve this.

This document details the primary interception techniques used across Linux and Windows, including how they work and their respective tradeoffs.

## Linux Interception

### 1. eBPF (Extended Berkeley Packet Filter)

The eBPF approach provides high-fidelity, kernel-level attribution of DNS traffic.

**How it works:**
- DNSFlow loads an eBPF program (`dnsflow-ebpf`) into the kernel using the `aya` library.
- A `kprobe` is attached to the `udp_sendmsg` kernel function.
- When any process attempts to send a UDP packet, the eBPF program executes.
- It retrieves the process ID (PID), Thread Group ID (TGID), and User ID (UID) using helper functions like `bpf_get_current_pid_tgid()`.
- This metadata, along with the destination address and port, is sent to the userspace DNSFlow daemon via a `PerfEventArray` (the `EVENTS` map).
- **Current Status**: In the current implementation, eBPF is used primarily for **query attribution and logging**. Transparent redirection via `cgroup_connect4` is currently a stub/experimental feature.

**Tradeoffs:**
- **Pros:** Extremely reliable attribution. Captures all UDP traffic, including from statically linked binaries (Go) and custom resolvers.
- **Cons:** Requires root privileges (`CAP_BPF`). Requires a modern Linux kernel (5.15+). Currently provides monitoring rather than redirection.

### 2. Mount Namespaces (Resolv.conf Bind-Mount)

This is the primary redirection technique for Linux applications launched via DNSFlow.

**How it works:**
- When an application is launched through the DNSFlow UI, it is wrapped in a new **Mount Namespace** using the `unshare` system call.
- Inside this private namespace, DNSFlow bind-mounts a custom `/etc/resolv.conf` file (located at `/tmp/dnsflow_resolv.conf`).
- This custom file points the application's default resolver to the local DNSFlow proxy (typically `127.0.0.53:53`).
- Because the mount is private to the application's namespace, the rest of the system remains unaffected.

**Tradeoffs:**
- **Pros:** Highly effective for applications that respect `/etc/resolv.conf` (most standard apps). No root required for the application itself (though `unshare` might require specific capabilities or be restricted by user namespaces).
- **Cons:** Does not affect applications with hardcoded DNS servers or those using DNS-over-HTTPS (DoH) unless specifically handled (see Browser Handling).

### 3. LD_PRELOAD Shim

A userspace technique that intercepts standard C library (libc) resolution calls.

**How it works:**
- DNSFlow provides a shared library (`dnsflow-shim`).
- The `LD_PRELOAD` environment variable is set to point to this library on launch.
- The shim overrides `getaddrinfo` and `gethostbyname`.
- It reads a rules configuration from `/tmp/dnsflow_rules.json` to determine if it should redirect queries to the local proxy.

**Tradeoffs:**
- **Pros:** Simple, no special permissions needed.
- **Cons:** Only works for dynamically linked applications using libc. Bypassed by Go apps and custom resolvers.

## Windows Interception

### 4. WinDivert (Windows Packet Divert)

WinDivert is a network-layer packet capture and modification framework for Windows.

**How it works:**
- DNSFlow opens a WinDivert handle with the filter `outbound and udp.DstPort == 53`.
- Outbound DNS packets are captured before they leave the network stack.
- DNSFlow uses `GetExtendedUdpTable` to look up the PID of the process owning the source port.
- If a rule matches, the destination IP is spoofed to `127.0.0.53` (the local proxy address).
- The modified packet is re-injected into the network stack.

**Tradeoffs:**
- **Pros:** Captures all DNS traffic regardless of application type.
- **Cons:** Requires a signed kernel driver (`WinDivert.sys`) and Administrator privileges.

### 5. DLL Injection (DNSAPI Hook)

A userspace API hooking technique for Windows.

**How it works:**
- DNSFlow injects `dnsflow-hook.dll` into the target process using `CreateRemoteThread`.
- The DLL hooks `DnsQuery_A` and `DnsQuery_W` in `DNSAPI.dll`.
- Intercepted queries are redirected to the local proxy.

**Tradeoffs:**
- **Pros:** Per-process isolation without a kernel driver.
- **Cons:** Often flagged by Antivirus/EDR. Limited to apps using the standard Windows DNS API.

## Process Ancestry Tracking

Across all techniques, DNSFlow uses **Ancestry Tracking** to ensure child processes inherit rules:
- When a process is launched, its PID is tracked in an `active_sessions` map.
- When the proxy receives a query, it walks up the process tree (up to 10 levels) to see if any ancestor is a tracked session.
- Additionally, the `DNSFLOW_RULE_ID` environment variable is injected into launched processes and checked during rule matching for robust attribution.
