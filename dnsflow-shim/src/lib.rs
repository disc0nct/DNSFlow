use libc::{addrinfo, c_int, dlsym, in_addr, sockaddr_in, AF_INET, RTLD_NEXT};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Once;

type GetAddrInfoFn = unsafe extern "C" fn(
    node: *const c_char,
    service: *const c_char,
    hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> c_int;

static INIT: Once = Once::new();
static mut ORIGINAL_GETADDRINFO: Option<GetAddrInfoFn> = None;

fn get_original_fn() -> GetAddrInfoFn {
    unsafe {
        INIT.call_once(|| {
            let sym_name = std::ffi::CString::new("getaddrinfo").unwrap();
            let sym_ptr = dlsym(RTLD_NEXT, sym_name.as_ptr());
            if !sym_ptr.is_null() {
                ORIGINAL_GETADDRINFO = Some(std::mem::transmute(sym_ptr));
            }
        });
        ORIGINAL_GETADDRINFO.unwrap()
    }
}

fn rules_file_path() -> String {
    std::env::var("DNSFLOW_RULES").unwrap_or_else(|_| {
        std::env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(|d| format!("{}/dnsflow_rules.json", d))
            .unwrap_or_else(|| "/tmp/dnsflow_rules.json".to_string())
    })
}

fn should_intercept() -> bool {
    let path = rules_file_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(rules) = serde_json::from_str::<serde_json::Value>(&content) {
            return rules["intercept"].as_bool().unwrap_or(false);
        }
    }
    false
}

fn get_proxy_addr() -> String {
    let path = rules_file_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(rules) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(addr) = rules["proxy_addr"].as_str() {
                return addr.to_string();
            }
        }
    }
    "127.0.0.53:53".to_string()
}

fn encode_dns_name(packet: &mut Vec<u8>, name: &str) {
    for label in name.split('.') {
        packet.push(label.len() as u8);
        packet.extend_from_slice(label.as_bytes());
    }
    packet.push(0);
}

fn build_dns_query(id: u16, hostname: &str) -> Vec<u8> {
    let mut packet = Vec::with_capacity(512);
    packet.extend_from_slice(&id.to_be_bytes());
    packet.extend_from_slice(&0x0100u16.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes());
    packet.extend_from_slice(&0u16.to_be_bytes());
    encode_dns_name(&mut packet, hostname);
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet
}

fn skip_dns_name(packet: &[u8], offset: usize) -> usize {
    let mut pos = offset;
    while pos < packet.len() {
        let len = packet[pos] as usize;
        if len == 0 {
            return pos + 1;
        }
        if (len & 0xC0) == 0xC0 {
            return pos + 2;
        }
        pos += len + 1;
    }
    pos
}

fn parse_dns_response(packet: &[u8]) -> Vec<std::net::Ipv4Addr> {
    if packet.len() < 12 {
        return vec![];
    }
    let ancount = u16::from_be_bytes([packet[6], packet[7]]) as usize;
    let mut ips = Vec::new();
    let mut offset = skip_dns_name(packet, 12);
    offset += 4;
    for _ in 0..ancount {
        if offset >= packet.len() {
            break;
        }
        offset = skip_dns_name(packet, offset);
        if offset + 10 > packet.len() {
            break;
        }
        let rtype = u16::from_be_bytes([packet[offset], packet[offset + 1]]);
        let rdlength = u16::from_be_bytes([packet[offset + 8], packet[offset + 9]]) as usize;
        offset += 10;
        if rtype == 1 && rdlength == 4 && offset + 4 <= packet.len() {
            ips.push(std::net::Ipv4Addr::new(
                packet[offset],
                packet[offset + 1],
                packet[offset + 2],
                packet[offset + 3],
            ));
        }
        offset += rdlength;
    }
    ips
}

fn dns_lookup(hostname: &str) -> Result<Vec<std::net::Ipv4Addr>, String> {
    use std::net::UdpSocket;

    let proxy_addr = get_proxy_addr();
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket
        .set_read_timeout(Some(std::time::Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;
    let query = build_dns_query(0x1234, hostname);
    socket
        .send_to(&query, &proxy_addr)
        .map_err(|e| e.to_string())?;
    let mut buf = [0u8; 512];
    let (len, _) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;
    Ok(parse_dns_response(&buf[..len]))
}

fn build_addrinfo_list(
    ips: Vec<std::net::Ipv4Addr>,
    hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> c_int {
    if ips.is_empty() {
        return libc::EAI_NONAME;
    }
    let mut head: *mut addrinfo = std::ptr::null_mut();
    let mut tail: *mut addrinfo = std::ptr::null_mut();
    for ip in ips {
        unsafe {
            let ptr = libc::malloc(std::mem::size_of::<addrinfo>()) as *mut addrinfo;
            if ptr.is_null() {
                free_addrinfo_list(head);
                return libc::EAI_MEMORY;
            }
            std::ptr::write_bytes(ptr, 0, 1);
            (*ptr).ai_flags = 0;
            (*ptr).ai_family = AF_INET;
            (*ptr).ai_socktype = if !hints.is_null() {
                (*hints).ai_socktype
            } else {
                libc::SOCK_STREAM
            };
            (*ptr).ai_protocol = if !hints.is_null() {
                (*hints).ai_protocol
            } else {
                0
            };
            (*ptr).ai_addrlen = std::mem::size_of::<sockaddr_in>() as u32;
            let sa = libc::malloc(std::mem::size_of::<sockaddr_in>()) as *mut sockaddr_in;
            if sa.is_null() {
                libc::free(ptr as *mut _);
                free_addrinfo_list(head);
                return libc::EAI_MEMORY;
            }
            std::ptr::write_bytes(sa, 0, 1);
            (*sa).sin_family = AF_INET as u16;
            (*sa).sin_port = 0;
            (*sa).sin_addr = in_addr {
                s_addr: u32::from(ip).to_be(),
            };
            (*ptr).ai_addr = sa as *mut libc::sockaddr;
            (*ptr).ai_next = std::ptr::null_mut();
            if head.is_null() {
                head = ptr;
                tail = ptr;
            } else {
                (*tail).ai_next = ptr;
                tail = ptr;
            }
        }
    }
    unsafe {
        *res = head;
    }
    0
}

unsafe fn free_addrinfo_list(mut head: *mut addrinfo) {
    while !head.is_null() {
        let next = (*head).ai_next;
        if !(*head).ai_addr.is_null() {
            libc::free((*head).ai_addr as *mut _);
        }
        libc::free(head as *mut _);
        head = next;
    }
}

#[no_mangle]
pub unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> c_int {
    let orig_fn = get_original_fn();

    if node.is_null() {
        return orig_fn(node, service, hints, res);
    }

    let hostname = CStr::from_ptr(node).to_string_lossy();

    if hostname == "localhost" || hostname == "127.0.0.1" || hostname == "::1" {
        return orig_fn(node, service, hints, res);
    }

    if !should_intercept() {
        return orig_fn(node, service, hints, res);
    }

    match dns_lookup(&hostname) {
        Ok(ips) if !ips.is_empty() => build_addrinfo_list(ips, hints, res),
        _ => orig_fn(node, service, hints, res),
    }
}
