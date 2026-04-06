/// DNS packet information extracted from network headers
#[derive(Debug, Clone)]
pub struct DnsPacketInfo {
    pub src_port: u16,
    pub dst_port: u16,
    pub dst_addr: std::net::Ipv4Addr,
    pub payload_offset: usize,
}

/// Parse DNS packet from raw bytes (Ethernet + IPv4 + UDP)
pub fn parse_dns_packet(buf: &[u8]) -> Result<DnsPacketInfo, &'static str> {
    if buf.len() < 42 {
        return Err("Packet too short");
    }

    // Parse Ethernet header (14 bytes)
    let ethertype = u16::from_be_bytes([buf[12], buf[13]]);
    if ethertype != 0x0800 {
        return Err("Not IPv4");
    }

    // Parse IPv4 header (20 bytes minimum)
    let ip_start = 14;
    let ip_version = (buf[ip_start] >> 4) & 0x0F;
    if ip_version != 4 {
        return Err("Not IPv4");
    }

    let ip_header_len = ((buf[ip_start] & 0x0F) * 4) as usize;
    if ip_header_len < 20 {
        return Err("Invalid IP header length (< 20 bytes)");
    }
    let protocol = buf[ip_start + 9];

    if protocol != 17 {
        return Err("Not UDP");
    }

    let dst_ip = std::net::Ipv4Addr::new(
        buf[ip_start + 16],
        buf[ip_start + 17],
        buf[ip_start + 18],
        buf[ip_start + 19],
    );

    // Parse UDP header (8 bytes)
    let udp_start = ip_start + ip_header_len;
    if buf.len() < udp_start + 8 {
        return Err("Packet too short for UDP");
    }

    let src_port = u16::from_be_bytes([buf[udp_start], buf[udp_start + 1]]);
    let dst_port = u16::from_be_bytes([buf[udp_start + 2], buf[udp_start + 3]]);

    if dst_port != 53 {
        return Err("Not DNS (port 53)");
    }

    Ok(DnsPacketInfo {
        src_port,
        dst_port,
        dst_addr: dst_ip,
        payload_offset: udp_start + 8,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_dns_packet() {
        let mut buf = vec![0u8; 64];

        // Ethernet header
        buf[12] = 0x08; // IPv4 ethertype
        buf[13] = 0x00;

        // IPv4 header
        buf[14] = 0x45; // Version 4, header length 20
        buf[23] = 17; // UDP protocol

        // UDP header
        buf[34] = 0x12; // Source port 0x1234
        buf[35] = 0x34;
        buf[36] = 0x00; // Dest port 53
        buf[37] = 0x35;

        let result = parse_dns_packet(&buf);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.src_port, 0x1234);
        assert_eq!(info.dst_port, 53);
    }

    #[test]
    fn test_parse_non_dns_returns_error() {
        let mut buf = vec![0u8; 64];
        buf[12] = 0x08;
        buf[13] = 0x00;
        buf[14] = 0x45;
        buf[23] = 17;
        buf[36] = 0x00; // Port 80, not 53
        buf[37] = 0x50;

        let result = parse_dns_packet(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_packet_too_short() {
        let buf = vec![0u8; 20];
        let result = parse_dns_packet(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_non_ipv4_packet() {
        let mut buf = vec![0u8; 64];
        buf[12] = 0x86; // ARP ethertype, not IPv4
        buf[13] = 0xDD;

        let result = parse_dns_packet(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_non_udp_packet() {
        let mut buf = vec![0u8; 64];
        buf[12] = 0x08;
        buf[13] = 0x00;
        buf[14] = 0x45;
        buf[23] = 6; // TCP protocol, not UDP

        let result = parse_dns_packet(&buf);
        assert!(result.is_err());
    }
}
