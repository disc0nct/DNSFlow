# DNSFlow Troubleshooting Guide

This guide helps you resolve common issues encountered while building, running, or routing DNS with DNSFlow.

## Build Issues

### SUDO_UID/SUDO_GID not set
* **Symptom:** Build fails with permission errors, or generated files have incorrect ownership when compiling the eBPF programs.
* **Cause:** Running build scripts or cargo via `sudo` without preserving the original user's ID environment variables. This causes generated artifacts to be owned by root, breaking subsequent non-root build steps.
* **Solution:** Export the environment variables manually before building, or use `sudo -E` to preserve the environment. 
  Example: `SUDO_UID=$(id -u) SUDO_GID=$(id -g) cargo build`

## Runtime Issues

### Port 53/5353 conflicts ("Address already in use")
* **Symptom:** The DNSFlow proxy server fails to start, displaying an `Address already in use (os error 98)` or similar socket binding error.
* **Cause:** Another service on your system (such as `systemd-resolved`, `dnsmasq`, `avahi-daemon`, or `bind`) is already listening on the DNSFlow proxy port (`127.0.0.53:53` or `127.0.0.1:5353`).
* **Solution:** 
  1. Identify the conflicting service: `sudo lsof -i :53` or `sudo netstat -tulpn | grep 53`.
  2. Stop the conflicting service (e.g., `sudo systemctl stop systemd-resolved`).
  3. Alternatively, change the proxy listen port in the DNSFlow configuration.

### WinDivert driver not found (Windows)
* **Symptom:** Network interception fails to initialize on Windows, with errors indicating that `WinDivert.dll` or `WinDivert.sys` cannot be found or loaded.
* **Cause:** The required WinDivert driver binaries are missing from the executable's directory, the app is not running with Administrator privileges, or an antivirus software quarantined the driver.
* **Solution:** 
  1. Ensure `WinDivert.dll` and `WinDivert64.sys` are present in the same directory as the DNSFlow executable.
  2. Relaunch DNSFlow as an Administrator.
  3. If your antivirus blocked the driver, add a folder exclusion for the DNSFlow directory.

## DNS Routing Issues

### Ancestry Tracking Limits
* **Symptom:** Deeply nested child processes (e.g., specific tabs or complex launcher scripts) fail to inherit DNS rules.
* **Cause:** The `RulesEngine` has a hard limit of **10 levels** of recursion when walking up the process tree (following `ppid`) to prevent infinite loops or performance degradation.
* **Solution:** If your application hierarchy is deeper than 10 levels, use the `DNSFLOW_RULE_ID` environment variable manually to force the rule, or simplify the launch sequence.

### WinDivert Offset Discrepancies (Windows)
* **Symptom:** Packet redirection on Windows is inconsistent or fails silently even when the driver is running.
* **Cause:** There is a known implementation detail where the packet parser expects an Ethernet header, but WinDivert's `Network` layer may provide packets starting at the IP header. This can cause redirection offsets (specifically for the destination IP at bytes 16-19) to be miscalculated.
* **Solution:** Ensure you are using the latest version of the `dnsflow-hook.dll` falling back to DLL injection if packet-level redirection is unstable on your specific network interface.

### LD_PRELOAD .so not found
* **Symptom:** Applications launched via DNSFlow fail to route their DNS queries properly and fall back to the system default DNS. Logs or terminal output may show `ERROR: ld.so: object '...dnsflow_shim.so' from LD_PRELOAD cannot be preloaded`.
* **Cause:** The `dnsflow-shim` shared library was not built, or the path configured for the LD_PRELOAD injector is incorrect.
* **Solution:** 
  1. Ensure you have built the shim library: `cargo build -p dnsflow-shim`.
  2. Verify that the path to `libdnsflow_shim.so` in your DNSFlow configuration/launcher points to the correct absolute path of the built library.

## Diagnostic Commands

If DNSFlow is not intercepting as expected, use these commands to verify its status:

### Linux (eBPF & Proxy)
- **Check eBPF status**: `sudo bpftool prog show | grep dnsflow`
- **Check proxy listener**: `sudo lsof -i -n -P | grep 53`
- **Check resolv.conf**: `cat /etc/resolv.conf` (Verify it points to `127.0.0.53`)
- **Check LD_PRELOAD path**: `ls -l target/debug/libdnsflow_shim.so`

### Windows (WinDivert & Proxy)
- **Check WinDivert driver**: `sc query windivert` (Status should be `RUNNING`)
- **Check proxy listener**: `netstat -ano | findstr :53`
- **Check hook DLL**: `ls target/x86_64-pc-windows-msvc/release/dnsflow_hook.dll`

## Debug Mode

If you are experiencing issues not listed above, enable Debug Mode to gain deeper visibility into DNSFlow's internal operations. Debug Mode provides verbose logging for proxy requests, eBPF events, and process matching.

**How to enable Debug Mode:**
* **Via UI:** Navigate to **Settings** → **Debug mode** and toggle it on.
* **Via Config:** Open your `config.json` file and set `"debug": true`.

## Getting Help

If you're still stuck after checking this guide and running in Debug Mode, please reach out for help:

* Open an issue on our [GitHub Issues](https://github.com/) page.
* When opening an issue, please include:
  1. Your operating system and version.
  2. The version of DNSFlow you are running.
  3. A copy of the application logs with **Debug Mode** enabled.
