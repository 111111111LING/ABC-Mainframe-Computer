use std::path::Path;

const IAP_HEADER_SIZE: usize = 512;

fn calculate_checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();
    for chunk in chunks {
        sum = sum.wrapping_add(u32::from_le_bytes(chunk.try_into().unwrap()));
    }
    if !remainder.is_empty() {
        let mut padded = [0u8; 4];
        padded[..remainder.len()].copy_from_slice(remainder);
        sum = sum.wrapping_add(u32::from_le_bytes(padded));
    }
    sum
}

#[tauri::command]
pub fn generate_firmware(firmware_path: String, output_path: String) -> Result<String, String> {
    let path = Path::new(&firmware_path);
    let firmware_data =
        std::fs::read(path).map_err(|e| format!("读取固件失败: {}", e))?;

    let total_len = IAP_HEADER_SIZE + firmware_data.len();
    let checksum = calculate_checksum(&firmware_data);

    let mut output = Vec::with_capacity(total_len);
    let mut header = vec![0u8; IAP_HEADER_SIZE];
    header[0..8].copy_from_slice(b"WCHNET\0\0");
    header[8..12].copy_from_slice(&(total_len as u32).to_le_bytes());
    header[12..16].copy_from_slice(&checksum.to_le_bytes());
    output.extend_from_slice(&header);
    output.extend_from_slice(&firmware_data);

    std::fs::write(&output_path, &output)
        .map_err(|e| format!("写入输出文件失败: {}", e))?;

    Ok(format!(
        "升级固件已生成: {}\n  文件大小: {} 字节\n  固件校验和: 0x{:08X}",
        output_path, total_len, checksum,
    ))
}
