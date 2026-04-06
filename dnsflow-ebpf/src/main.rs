#![no_std]
#![no_main]

use aya_ebpf::{
    bindings::bpf_sock_addr,
    helpers::{bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_get_current_uid_gid},
    macros::{cgroup_connect4, kprobe, map},
    maps::{HashMap, PerfEventArray},
    programs::ProbeContext,
};
use aya_log_ebpf::info;
use dnsflow_common::DnsEvent;

#[map]
pub static mut EVENTS: PerfEventArray<DnsEvent> = PerfEventArray::with_max_entries(1024, 0);

#[map]
pub static mut RULES: HashMap<u32, u32> = HashMap::with_max_entries(1024, 0);

#[cgroup_connect4]
pub fn dnsflow_cgroup_connect(ctx: *mut bpf_sock_addr) -> i32 {
    // For MVP, this is a stub.
    // In a real implementation:
    // 1. Get tgid from bpf_get_current_pid_tgid()
    // 2. Look up tgid in RULES map
    // 3. If found, check if dport == 53 (DNS)
    // 4. If so, redirect to 127.0.0.1:5353

    // Just pass for now
    1 // 1 means BPF_CGROUP_INET4_CONNECT_PROCEED
}

#[kprobe]
pub fn dnsflow_kprobe(ctx: ProbeContext) -> u32 {
    match try_dnsflow_kprobe(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_dnsflow_kprobe(ctx: ProbeContext) -> Result<u32, u32> {
    let pid_tgid = bpf_get_current_pid_tgid();
    let uid_gid = bpf_get_current_uid_gid();

    let mut event = DnsEvent {
        pid: (pid_tgid >> 32) as u32,
        tgid: pid_tgid as u32,
        uid: (uid_gid >> 32) as u32,
        gid: uid_gid as u32,
        comm: [0; 16],
        daddr: [0; 4],
        dport: 0,
        is_dns: false,
    };

    // Get command name
    if let Ok(comm) = bpf_get_current_comm() {
        event.comm = comm;
    }

    // In a real implementation, we would extract the sock struct and check if dport == 53
    // For this MVP/stub, we just emit an event to prove the pipeline works
    event.is_dns = true;
    event.dport = 53;

    // Emit to userspace
    unsafe {
        EVENTS.output(&ctx, &event, 0);
    }

    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
