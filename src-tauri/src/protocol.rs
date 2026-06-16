pub const FRAME_H1: u8 = 0x5A;
pub const FRAME_H2: u8 = 0xA5;

pub const CMD_CFG_DEVICE_ID: u8 = 0x01;
pub const CMD_CFG_NETWORK: u8 = 0x02;
pub const CMD_OTA_FRONT: u8 = 0x03;
pub const CMD_OTA_BACK: u8 = 0x04;
pub const CMD_IAP_317: u8 = 0x05;

// fota_t 包结构 (对应 8018 的 fota_t)
const FOTA_HEADER_SIZE: usize = 20;
const AEAD_TAG_LEN: usize = 16;

/// 从 .fota 文件中解析出一个个独立的 fota_t 包
pub fn split_fota_packets(data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut packets = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        if offset + FOTA_HEADER_SIZE > data.len() {
            return Err(format!("fota文件截断: offset={} < header", offset));
        }
        let op = data[offset + 14]; // fota_t.operation
        let size = u32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()); // fota_t.size
        let pkt_len = match op {
            0xEE => FOTA_HEADER_SIZE + 12 + 32, // 64
            0xFF => FOTA_HEADER_SIZE + size as usize + AEAD_TAG_LEN,
            0xCC => FOTA_HEADER_SIZE + 20, // 40
            _ => return Err(format!("未知fota操作: 0x{:02X} @ offset {}", op, offset)),
        };
        if offset + pkt_len > data.len() {
            return Err(format!(
                "fota包截断: offset={}, need={}, have={}",
                offset,
                pkt_len,
                data.len() - offset
            ));
        }
        packets.push(data[offset..offset + pkt_len].to_vec());
        offset += pkt_len;
    }
    Ok(packets)
}

pub fn build_frame(cmd: u8, data: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(5 + data.len());
    frame.push(FRAME_H1);
    frame.push(FRAME_H2);
    frame.push(cmd);
    frame.push((data.len() & 0xFF) as u8);
    frame.push(((data.len() >> 8) & 0xFF) as u8);
    frame.extend_from_slice(data);
    frame
}

pub fn find_ack(buf: &[u8]) -> Option<(usize, u8, u8)> {
    if buf.len() < 6 {
        return None;
    }
    for i in 0..=buf.len() - 6 {
        if buf[i] == FRAME_H1 && buf[i + 1] == FRAME_H2 {
            let resp_cmd = buf[i + 2];
            let status_len = buf[i + 3] as u16 | ((buf[i + 4] as u16) << 8);
            let total = 5 + status_len as usize;
            if i + total <= buf.len() {
                let status = if status_len > 0 { buf[i + 5] } else { 0xFF };
                return Some((i + total, resp_cmd, status));
            }
        }
    }
    None
}

pub fn pack_device_identity(config: &crate::config::DeviceConfig) -> Vec<u8> {
    let mut data = Vec::with_capacity(129);
    let mut buf = [0u8; 32];
    let pid = config.product_id.as_bytes();
    let len = pid.len().min(32);
    buf[..len].copy_from_slice(&pid[..len]);
    data.extend_from_slice(&buf);
    buf.fill(0);
    let dn = config.device_name.as_bytes();
    let len = dn.len().min(32);
    buf[..len].copy_from_slice(&dn[..len]);
    data.extend_from_slice(&buf);
    let mut buf64 = [0u8; 64];
    let sk = config.sec_key.as_bytes();
    let len = sk.len().min(64);
    buf64[..len].copy_from_slice(&sk[..len]);
    data.extend_from_slice(&buf64);
    data.push(config.bind);
    data
}

pub fn pack_network_config(config: &crate::config::NetworkConfig) -> Vec<u8> {
    let mut data = Vec::with_capacity(31);
    data.push(config.lan_dhcp as u8);
    data.extend_from_slice(&parse_ip(&config.lan_ip));
    data.extend_from_slice(&parse_ip(&config.lan_mask));
    data.extend_from_slice(&parse_ip(&config.lan_gateway));
    data.extend_from_slice(&parse_mac(&config.mac_addr));
    data.extend_from_slice(&parse_ip(&config.mqtt_server_ip));
    data.extend_from_slice(&config.mqtt_server_port.to_le_bytes());
    data.extend_from_slice(&parse_ip(&config.ntp_server));
    data.extend_from_slice(&config.ntp_port.to_le_bytes());
    data
}

fn parse_ip(s: &str) -> [u8; 4] {
    let parts: Vec<u8> = s.split('.').filter_map(|p| p.parse().ok()).collect();
    let mut ip = [0u8; 4];
    for (i, &v) in parts.iter().enumerate().take(4) {
        ip[i] = v;
    }
    ip
}

fn parse_mac(s: &str) -> [u8; 6] {
    let hex: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let mut mac = [0u8; 6];
    for i in 0..6 {
        if i * 2 + 1 < hex.len() {
            mac[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap_or(0);
        }
    }
    mac
}
