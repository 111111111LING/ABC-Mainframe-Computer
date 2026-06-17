#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod auth;
mod config;
mod database;
mod excel;
mod firmware;
mod iap_server;
mod protocol;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            iap_server::init(&app.handle());
            let state = database::ConfigState::new(&app.handle());
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            config::save_config,
            config::load_config,
            config::get_default_config,
            firmware::generate_firmware,
            iap_server::start_tcp_server,
            iap_server::stop_tcp_server,
            iap_server::get_server_status,
            iap_server::send_device_config,
            iap_server::send_network_config,
            iap_server::start_iap_upgrade,
            iap_server::start_ota_upgrade,
            database::get_all_devices,
            database::search_devices,
            database::save_device_config,
            database::delete_device,
            database::set_excel_imported,
            excel::import_excel,
            excel::export_excel,
            auth::is_password_set,
            auth::set_admin_password,
            auth::verify_admin_password,
            auth::reset_admin_password,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
