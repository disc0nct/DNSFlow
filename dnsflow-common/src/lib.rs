#![no_std]

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DnsEvent {
    pub pid: u32,
    pub tgid: u32,
    pub uid: u32,
    pub gid: u32,
    pub comm: [u8; 16],
    pub daddr: [u8; 4], // IPv4 for now
    pub dport: u16,
    pub is_dns: bool,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for DnsEvent {}