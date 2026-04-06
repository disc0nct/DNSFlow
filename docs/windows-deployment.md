# Windows Deployment Guide

This document outlines the specific requirements and considerations for deploying DNSFlow on Windows 10 and 11.

## Interception Methods

DNSFlow on Windows uses two primary techniques to redirect DNS traffic to its local proxy:
1. **WinDivert (Packet-level)**: For system-wide/statically-linked application capture.
2. **DNSAPI Hook (Per-process)**: For specific application redirection without a kernel driver.

## WinDivert Driver Requirement

DNSFlow uses [WinDivert](https://reqrypt.org/windivert.html) for packet interception. For the application to function correctly, the signed `WinDivert.sys` driver must be present in the same directory as the application executable.

- **Files Needed**: `WinDivert.sys`, `WinDivert.dll`.
- **Permissions**: WinDivert requires **Administrative privileges** to load the driver and capture packets.

## DNSAPI Hook DLL

The per-process DNS override is achieved by injecting `dnsflow_hook.dll` into target processes. This DLL is part of the DNSFlow project (`dnsflow-hook` crate).

- **Build Instructions**:
  ```powershell
  cargo build -p dnsflow-hook --release --target x86_64-pc-windows-msvc
  ```
  The resulting DLL will be located in `target/x86_64-pc-windows-msvc/release/dnsflow_hook.dll`.

## UAC Elevation

DNSFlow requires Administrative privileges on Windows to perform low-level operations.

- **Elevation Logic**: On startup, the Rust backend calls `ensure_elevated()`.
- **Mechanism**: If not elevated, it uses `ShellExecuteExW` with the `runas` verb to trigger a UAC prompt and relaunch itself.

## Antivirus (AV) Considerations

DNSFlow uses techniques that are often flagged by Antivirus software as potentially malicious:
1. **Packet Interception**: WinDivert's driver might be blocked.
2. **DLL Injection**: `CreateRemoteThread` and `LoadLibrary` into target processes is a common AV trigger.

### Recommendations for Deployment
- **Folder Exclusion**: Strongly recommended to add the DNSFlow installation directory to the AV exclusion list.
- **Code Signing**: Ensure that the main executable and all DLLs (`WinDivert.dll`, `dnsflow_hook.dll`) are signed with an EV certificate to improve reputation.

## Packaging with Tauri

Update your `tauri.conf.json` to include the necessary resources in the `bundle` section:

```json
{
  "bundle": {
    "resources": [
      "binaries/WinDivert.dll",
      "binaries/WinDivert.sys",
      "binaries/dnsflow_hook.dll"
    ]
  }
}
```

### Build Command
```bash
npm run tauri build -- --target x86_64-pc-windows-msvc
```
