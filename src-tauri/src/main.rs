#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod firmware;
mod iap_server;
mod protocol;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            iap_server::init(&app.handle());
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
