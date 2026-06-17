use std::fs::File;
use std::io::BufReader;

use calamine::{open_workbook, Reader, Xlsx};
use rust_xlsxwriter::*;

use crate::config::DeviceRecord;

#[tauri::command]
pub fn import_excel(path: String) -> Result<Vec<DeviceRecord>, String> {
    let mut workbook = open_workbook::<Xlsx<BufReader<File>>, _>(&path)
        .map_err(|e| e.to_string())?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .ok_or("Excel文件没有工作表")?
        .clone();
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| e.to_string())?;

    let mut records = Vec::new();
    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            continue;
        }
        if row.len() < 13 {
            continue;
        }

        let to_str = |idx: usize| -> String {
            row.get(idx)
                .map(|v| v.to_string().trim().to_string())
                .unwrap_or_default()
        };
        let to_bool = |idx: usize| -> bool {
            let s = to_str(idx).to_lowercase();
            s == "1" || s == "true" || s == "yes"
        };
        let to_u16 = |idx: usize| -> u16 {
            to_str(idx).parse::<u16>().unwrap_or(0)
        };

        records.push(DeviceRecord {
            id: None,
            product_id: to_str(0),
            device_name: to_str(1),
            sec_key: to_str(2),
            bind: to_bool(3),
            lan_dhcp: to_bool(4),
            lan_ip: to_str(5),
            lan_gateway: to_str(6),
            lan_mask: to_str(7),
            mac_addr: to_str(8),
            mqtt_domain: to_str(9),
            mqtt_port: to_u16(10),
            ntp_ip: to_str(11),
            ntp_port: to_u16(12),
            configured: false,
            created_at: None,
            configured_at: None,
        });
    }

    Ok(records)
}

#[tauri::command]
pub fn export_excel(path: String, records: Vec<DeviceRecord>) -> Result<(), String> {
    let mut workbook = Workbook::new();

    let sheet = workbook.add_worksheet();

    let col = |idx| idx as u16;

    let headers = [
        "ProductID",
        "DeviceName",
        "SecKey",
        "Bind",
        "DHCP",
        "IP",
        "Gateway",
        "Mask",
        "MAC",
        "MQTT_Domain",
        "MQTT_Port",
        "NTP_IP",
        "NTP_Port",
        "已配置",
    ];
    for (i, h) in headers.iter().enumerate() {
        sheet
            .write_string(0, col(i), *h)
            .map_err(|e| e.to_string())?;
    }

    for (i, rec) in records.iter().enumerate() {
        let r = (i + 1) as u32;
        sheet
            .write_string(r, col(0), &rec.product_id)
            .map_err(|e| e.to_string())?;
        sheet
            .write_string(r, col(1), &rec.device_name)
            .map_err(|e| e.to_string())?;
        sheet
            .write_string(r, col(2), &rec.sec_key)
            .map_err(|e| e.to_string())?;
        sheet
            .write_number(r, col(3), rec.bind as i32)
            .map_err(|e| e.to_string())?;
        sheet
            .write_number(r, col(4), rec.lan_dhcp as i32)
            .map_err(|e| e.to_string())?;
        sheet
            .write_string(r, col(5), &rec.lan_ip)
            .map_err(|e| e.to_string())?;
        sheet
            .write_string(r, col(6), &rec.lan_gateway)
            .map_err(|e| e.to_string())?;
        sheet
            .write_string(r, col(7), &rec.lan_mask)
            .map_err(|e| e.to_string())?;
        sheet
            .write_string(r, col(8), &rec.mac_addr)
            .map_err(|e| e.to_string())?;
        sheet
            .write_string(r, col(9), &rec.mqtt_domain)
            .map_err(|e| e.to_string())?;
        sheet
            .write_number(r, col(10), rec.mqtt_port as i32)
            .map_err(|e| e.to_string())?;
        sheet
            .write_string(r, col(11), &rec.ntp_ip)
            .map_err(|e| e.to_string())?;
        sheet
            .write_number(r, col(12), rec.ntp_port as i32)
            .map_err(|e| e.to_string())?;
        sheet
            .write_string(r, col(13), if rec.configured { "已配置" } else { "未配置" })
            .map_err(|e| e.to_string())?;
    }

    workbook.save(&path).map_err(|e| e.to_string())?;
    Ok(())
}
