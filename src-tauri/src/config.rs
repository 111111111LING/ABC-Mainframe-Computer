use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub product_id: String,
    pub device_name: String,
    pub sec_key: String,
    pub bind: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub lan_dhcp: bool,
    pub lan_ip: String,
    pub lan_gateway: String,
    pub lan_mask: String,
    pub mac_addr: String,
    pub mqtt_server_ip: String,
    pub mqtt_server_port: u16,
    pub ntp_server: String,
    pub ntp_port: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            lan_dhcp: true,
            lan_ip: "192.168.124.100".into(),
            lan_gateway: "192.168.124.1".into(),
            lan_mask: "255.255.255.0".into(),
            mac_addr: "04:2B:58:09:D2:F3".into(),
            mqtt_server_ip: "101.132.160.111".into(),
            mqtt_server_port: 8883,
            ntp_server: "106.14.18.202".into(),
            ntp_port: 12123,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub device_config: DeviceConfig,
    pub network_config: NetworkConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_config: DeviceConfig {
                product_id: "7KdEHCyUDG".into(),
                device_name: "DVDR260512010901".into(),
                sec_key: "N0Y0NzY2NDYyQzI2MkU0MjAwRjZCNTEwQkRCMkI4MkU=".into(),
                bind: 1,
            },
            network_config: NetworkConfig::default(),
        }
    }
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<String, String> {
    serde_json::to_string_pretty(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_config(path: String) -> Result<AppConfig, String> {
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析失败: {}", e))
}

#[tauri::command]
pub fn get_default_config() -> AppConfig {
    AppConfig::default()
}
