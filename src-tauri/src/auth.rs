use std::fs;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::database::ConfigState;

#[derive(Debug, Serialize, Deserialize)]
struct PasswordData {
    hash: String,
}

fn read_password_data(path: &std::path::Path) -> Result<Option<PasswordData>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| format!("读取密码文件失败: {}", e))?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|e| format!("解析密码文件失败: {}", e))
}

fn write_password_data(path: &std::path::Path, data: &PasswordData) -> Result<(), String> {
    let content =
        serde_json::to_string_pretty(data).map_err(|e| format!("序列化密码失败: {}", e))?;
    fs::write(path, content).map_err(|e| format!("写入密码文件失败: {}", e))
}

#[tauri::command]
pub fn is_password_set(state: State<'_, ConfigState>) -> Result<bool, String> {
    let path = &state.config_path;
    let data = read_password_data(path)?;
    Ok(data.is_some())
}

#[tauri::command]
pub fn set_admin_password(state: State<'_, ConfigState>, password: String) -> Result<(), String> {
    if password.is_empty() {
        return Err("密码不能为空".into());
    }
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("哈希密码失败: {}", e))?
        .to_string();
    let data = PasswordData { hash };
    write_password_data(&state.config_path, &data)
}

#[tauri::command]
pub fn reset_admin_password(state: State<'_, ConfigState>) -> Result<(), String> {
    let path = &state.config_path;
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| format!("删除密码文件失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn verify_admin_password(
    state: State<'_, ConfigState>,
    password: String,
) -> Result<bool, String> {
    let data = read_password_data(&state.config_path)?;
    match data {
        None => Err("密码未设置".into()),
        Some(d) => {
            let parsed_hash =
                PasswordHash::new(&d.hash).map_err(|e| format!("解析密码哈希失败: {}", e))?;
            Ok(Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok())
        }
    }
}
