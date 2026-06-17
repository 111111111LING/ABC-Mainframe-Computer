use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, State};

use crate::config::DeviceRecord;

pub struct ConfigState {
    pub db: Mutex<Connection>,
    pub session_records: Mutex<Vec<DeviceRecord>>,
    pub excel_imported: Mutex<bool>,
    pub config_path: PathBuf,
}

impl ConfigState {
    pub fn new(app: &AppHandle) -> Self {
        let config_dir = app
            .path_resolver()
            .app_config_dir()
            .expect("Failed to get config dir");
        std::fs::create_dir_all(&config_dir).ok();
        let db_path = config_dir.join("devices.db");
        let db = Connection::open(&db_path).expect("Failed to open database");

        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS devices (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                product_id TEXT NOT NULL DEFAULT '',
                device_name TEXT NOT NULL DEFAULT '',
                sec_key TEXT NOT NULL DEFAULT '',
                bind INTEGER NOT NULL DEFAULT 1,
                lan_dhcp INTEGER NOT NULL DEFAULT 1,
                lan_ip TEXT NOT NULL DEFAULT '',
                lan_gateway TEXT NOT NULL DEFAULT '',
                lan_mask TEXT NOT NULL DEFAULT '',
                mac_addr TEXT NOT NULL DEFAULT '',
                mqtt_domain TEXT NOT NULL DEFAULT '',
                mqtt_port INTEGER NOT NULL DEFAULT 0,
                ntp_ip TEXT NOT NULL DEFAULT '',
                ntp_port INTEGER NOT NULL DEFAULT 0,
                configured INTEGER NOT NULL DEFAULT 0,
                created_at TEXT DEFAULT (datetime('now','localtime')),
                configured_at TEXT
            )",
        )
        .expect("Failed to create table");

        Self {
            db: Mutex::new(db),
            session_records: Mutex::new(Vec::new()),
            excel_imported: Mutex::new(false),
            config_path: config_dir.join("config.json"),
        }
    }
}

fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<DeviceRecord> {
    Ok(DeviceRecord {
        id: Some(row.get(0)?),
        product_id: row.get(1)?,
        device_name: row.get(2)?,
        sec_key: row.get(3)?,
        bind: row.get::<_, i32>(4)? != 0,
        lan_dhcp: row.get::<_, i32>(5)? != 0,
        lan_ip: row.get(6)?,
        lan_gateway: row.get(7)?,
        lan_mask: row.get(8)?,
        mac_addr: row.get(9)?,
        mqtt_domain: row.get(10)?,
        mqtt_port: row.get::<_, i32>(11)? as u16,
        ntp_ip: row.get(12)?,
        ntp_port: row.get::<_, i32>(13)? as u16,
        configured: row.get::<_, i32>(14)? != 0,
        created_at: row.get(15)?,
        configured_at: row.get(16)?,
    })
}

pub fn get_all_from_db(db: &Connection) -> Result<Vec<DeviceRecord>, String> {
    let mut stmt = db
        .prepare("SELECT * FROM devices ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], row_to_record)
        .map_err(|e| e.to_string())?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row.map_err(|e| e.to_string())?);
    }
    Ok(records)
}

pub fn search_name_in_db(db: &Connection, kw: &str) -> Result<Vec<DeviceRecord>, String> {
    let pattern = format!("%{}%", kw);
    let mut stmt = db
        .prepare("SELECT * FROM devices WHERE device_name LIKE ?1 ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([&pattern], row_to_record)
        .map_err(|e| e.to_string())?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row.map_err(|e| e.to_string())?);
    }
    Ok(records)
}

pub fn search_ip_in_db(db: &Connection, kw: &str) -> Result<Vec<DeviceRecord>, String> {
    let pattern = format!("%{}%", kw);
    let mut stmt = db
        .prepare("SELECT * FROM devices WHERE lan_ip LIKE ?1 ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([&pattern], row_to_record)
        .map_err(|e| e.to_string())?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row.map_err(|e| e.to_string())?);
    }
    Ok(records)
}

pub fn find_by_name_in_db(db: &Connection, name: &str) -> Result<Option<DeviceRecord>, String> {
    let mut stmt = db
        .prepare("SELECT * FROM devices WHERE device_name = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map([name], row_to_record)
        .map_err(|e| e.to_string())?;
    match rows.next() {
        Some(row) => Ok(Some(row.map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

pub fn insert_device_in_db(db: &Connection, record: &DeviceRecord) -> Result<i64, String> {
    db.execute(
        "INSERT INTO devices (product_id, device_name, sec_key, bind, lan_dhcp, lan_ip, lan_gateway, lan_mask, mac_addr, mqtt_domain, mqtt_port, ntp_ip, ntp_port, configured)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0)",
        rusqlite::params![
            record.product_id,
            record.device_name,
            record.sec_key,
            record.bind as i32,
            record.lan_dhcp as i32,
            record.lan_ip,
            record.lan_gateway,
            record.lan_mask,
            record.mac_addr,
            record.mqtt_domain,
            record.mqtt_port as i32,
            record.ntp_ip,
            record.ntp_port as i32,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(db.last_insert_rowid())
}

pub fn update_device_in_db(db: &Connection, id: i64, record: &DeviceRecord) -> Result<(), String> {
    db.execute(
        "UPDATE devices SET product_id=?1, device_name=?2, sec_key=?3, bind=?4, lan_dhcp=?5, lan_ip=?6, lan_gateway=?7, lan_mask=?8, mac_addr=?9, mqtt_domain=?10, mqtt_port=?11, ntp_ip=?12, ntp_port=?13 WHERE id=?14",
        rusqlite::params![
            record.product_id,
            record.device_name,
            record.sec_key,
            record.bind as i32,
            record.lan_dhcp as i32,
            record.lan_ip,
            record.lan_gateway,
            record.lan_mask,
            record.mac_addr,
            record.mqtt_domain,
            record.mqtt_port as i32,
            record.ntp_ip,
            record.ntp_port as i32,
            id,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn mark_configured_in_db(db: &Connection, id: i64) -> Result<(), String> {
    db.execute(
        "UPDATE devices SET configured=1, configured_at=datetime('now','localtime') WHERE id=?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_device_from_db(db: &Connection, id: i64) -> Result<(), String> {
    db.execute("DELETE FROM devices WHERE id=?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn filter_session_by_name(records: &[DeviceRecord], kw: &str) -> Vec<DeviceRecord> {
    if kw.is_empty() {
        return records.to_vec();
    }
    let kw = kw.to_lowercase();
    records
        .iter()
        .filter(|r| r.device_name.to_lowercase().contains(&kw))
        .cloned()
        .collect()
}

fn filter_session_by_ip(records: &[DeviceRecord], kw: &str) -> Vec<DeviceRecord> {
    if kw.is_empty() {
        return records.to_vec();
    }
    let kw = kw.to_lowercase();
    records
        .iter()
        .filter(|r| r.lan_ip.to_lowercase().contains(&kw))
        .cloned()
        .collect()
}

#[tauri::command]
pub fn get_all_devices(state: State<'_, ConfigState>) -> Result<Vec<DeviceRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut records = get_all_from_db(&db)?;
    let session = state.session_records.lock().map_err(|e| e.to_string())?;
    records.extend(session.clone());
    Ok(records)
}

#[tauri::command]
pub fn search_devices(
    state: State<'_, ConfigState>,
    name_kw: String,
    ip_kw: String,
) -> Result<Vec<DeviceRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let from_db = if !name_kw.is_empty() {
        search_name_in_db(&db, &name_kw)?
    } else if !ip_kw.is_empty() {
        search_ip_in_db(&db, &ip_kw)?
    } else {
        get_all_from_db(&db)?
    };

    let mut from_db = from_db;
    let session = state.session_records.lock().map_err(|e| e.to_string())?;

    let mut filtered_session = session.clone();
    if !name_kw.is_empty() {
        filtered_session = filter_session_by_name(&filtered_session, &name_kw);
    }
    if !ip_kw.is_empty() {
        filtered_session = filter_session_by_ip(&filtered_session, &ip_kw);
    }

    from_db.extend(filtered_session);
    Ok(from_db)
}

#[tauri::command]
pub fn set_excel_imported(state: State<'_, ConfigState>, imported: bool) -> Result<(), String> {
    let mut flag = state.excel_imported.lock().map_err(|e| e.to_string())?;
    *flag = imported;
    Ok(())
}

#[tauri::command]
pub async fn save_device_config(
    app_handle: AppHandle,
    state: State<'_, ConfigState>,
    record: DeviceRecord,
) -> Result<String, String> {
    let existing = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        find_by_name_in_db(&db, &record.device_name)?
    };

    let should_insert = match &existing {
        Some(prev) => prev.product_id != record.product_id || prev.sec_key != record.sec_key,
        None => true,
    };

    {
        let excel_imported = *state.excel_imported.lock().map_err(|e| e.to_string())?;
        if excel_imported {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            if should_insert {
                insert_device_in_db(&db, &record)?;
            } else if let Some(prev) = &existing {
                update_device_in_db(&db, prev.id.unwrap(), &record)?;
            }
        } else {
            let mut session = state.session_records.lock().map_err(|e| e.to_string())?;
            session.push(record.clone());
        }
    }

    let dev_cfg = crate::config::DeviceConfig {
        product_id: record.product_id.clone(),
        device_name: record.device_name.clone(),
        sec_key: record.sec_key.clone(),
        bind: if record.bind { 1 } else { 0 },
    };
    let data = crate::protocol::pack_device_identity(&dev_cfg);
    let frame = crate::protocol::build_frame(crate::protocol::CMD_CFG_DEVICE_ID, &data);
    crate::iap_server::send_frame_and_wait(&app_handle, frame, crate::protocol::CMD_CFG_DEVICE_ID | 0x80).await?;

    let net_cfg = crate::config::NetworkConfig {
        lan_dhcp: record.lan_dhcp,
        lan_ip: record.lan_ip.clone(),
        lan_gateway: record.lan_gateway.clone(),
        lan_mask: record.lan_mask.clone(),
        mac_addr: record.mac_addr.clone(),
        mqtt_server_ip: record.mqtt_domain.clone(),
        mqtt_server_port: record.mqtt_port,
        ntp_server: record.ntp_ip.clone(),
        ntp_port: record.ntp_port,
    };
    let data = crate::protocol::pack_network_config(&net_cfg);
    let frame = crate::protocol::build_frame(crate::protocol::CMD_CFG_NETWORK, &data);
    crate::iap_server::send_frame_and_wait(&app_handle, frame, crate::protocol::CMD_CFG_NETWORK | 0x80).await?;

    if let Some(id) = record.id {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        mark_configured_in_db(&db, id)?;
    }

    Ok("配置已保存并发送到设备".into())
}

#[tauri::command]
pub fn delete_device(state: State<'_, ConfigState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    delete_device_from_db(&db, id)
}
