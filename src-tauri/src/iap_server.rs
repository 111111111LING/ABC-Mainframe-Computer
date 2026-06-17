use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{Duration, Instant};

use crate::protocol;

const TCP_PORT: u16 = 1000;
const ACK_TIMEOUT_MS: u64 = 5000;

enum ServerCmd {
    SendAndWaitAck {
        frame: Vec<u8>,
        expected_resp_cmd: u8,
        resp_tx: oneshot::Sender<Result<u8, String>>,
    },
    SendRaw {
        data: Vec<u8>,
        resp_tx: oneshot::Sender<Result<(), String>>,
    },
    Shutdown,
}

struct ServerInner {
    running: bool,
    connected: bool,
    cmd_tx: Option<mpsc::Sender<ServerCmd>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

pub struct IapState {
    inner: Mutex<ServerInner>,
}

impl IapState {
    fn new() -> Self {
        Self {
            inner: Mutex::new(ServerInner {
                running: false,
                connected: false,
                cmd_tx: None,
                shutdown_tx: None,
            }),
        }
    }
}

pub fn init(app: &AppHandle) {
    app.manage(IapState::new());
}

fn emit_event(app: &AppHandle, event: &str, data: &str) {
    let _ = app.emit_all(event, data);
}

fn emit_log(app: &AppHandle, msg: &str) {
    emit_event(app, "iap-log", msg);
}

#[tauri::command]
pub async fn start_tcp_server(app_handle: AppHandle) -> Result<String, String> {
    let state = app_handle.state::<IapState>();
    {
        let s = state.inner.lock().unwrap();
        if s.running {
            return Err("服务器已在运行".into());
        }
    }

    let (cmd_tx, cmd_rx) = mpsc::channel::<ServerCmd>(32);
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    {
        let mut s = state.inner.lock().unwrap();
        s.cmd_tx = Some(cmd_tx);
        s.shutdown_tx = Some(shutdown_tx);
        s.running = true;
        s.connected = false;
    }

    let app = app_handle.clone();
    tokio::spawn(async move {
        let listener = match TcpListener::bind(format!("0.0.0.0:{}", TCP_PORT)).await {
            Ok(l) => l,
            Err(e) => {
                emit_log(&app, &format!("ERROR: 绑定端口{}失败: {}", TCP_PORT, e));
                let st = app.state::<IapState>();
                st.inner.lock().unwrap().running = false;
                return;
            }
        };

        emit_log(&app, &format!("TCP Server 已启动在 :{}", TCP_PORT));

        let mut stream = tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        emit_log(&app, &format!("设备已连接: {}", addr));
                        emit_event(&app, "device-connected", &addr.to_string());
                        stream
                    }
                    Err(e) => {
                        emit_log(&app, &format!("ERROR: 接受连接失败: {}", e));
                        let st = app.state::<IapState>();
                        st.inner.lock().unwrap().running = false;
                        return;
                    }
                }
            }
            _ = shutdown_rx => {
                emit_log(&app, "TCP Server 已停止");
                let st = app.state::<IapState>();
                st.inner.lock().unwrap().running = false;
                return;
            }
        };

        {
            let st = app.state::<IapState>();
            st.inner.lock().unwrap().connected = true;
        }

        let mut recv_buf = Vec::new();
        let mut tmp = [0u8; 4096];
        let mut cmd_rx = cmd_rx;

        loop {
            tokio::select! {
                result = stream.read(&mut tmp) => {
                    match result {
                        Ok(0) => {
                            emit_log(&app, "设备已断开连接");
                            emit_event(&app, "device-disconnected", "");
                            break;
                        }
                        Ok(n) => {
                            recv_buf.extend_from_slice(&tmp[..n]);
                            while let Some((consumed, resp_cmd, status)) = protocol::find_ack(&recv_buf) {
                                let msg = if status == 0 {
                                    format!("收到ACK: cmd=0x{:02X}", resp_cmd & 0x7F)
                                } else {
                                    format!("收到NACK: cmd=0x{:02X} status=0x{:02X}", resp_cmd & 0x7F, status)
                                };
                                emit_log(&app, &msg);
                                recv_buf.drain(..consumed);
                            }
                        }
                        Err(e) => {
                            emit_log(&app, &format!("读取错误: {}", e));
                            break;
                        }
                    }
                }
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(ServerCmd::SendAndWaitAck { frame, expected_resp_cmd, resp_tx }) => {
                            if let Err(e) = stream.write_all(&frame).await {
                                let _ = resp_tx.send(Err(format!("发送失败: {}", e)));
                                emit_log(&app, &format!("ERROR: 发送失败: {}", e));
                                break;
                            }
                            let result = wait_for_ack(&mut stream, &mut recv_buf, expected_resp_cmd, ACK_TIMEOUT_MS).await;
                            let _ = resp_tx.send(result);
                        }
                        Some(ServerCmd::SendRaw { data, resp_tx }) => {
                            if let Err(e) = stream.write_all(&data).await {
                                let _ = resp_tx.send(Err(format!("发送失败: {}", e)));
                                emit_log(&app, &format!("ERROR: 发送失败: {}", e));
                                break;
                            }
                            let _ = resp_tx.send(Ok(()));
                        }
                        Some(ServerCmd::Shutdown) | None => break,
                    }
                }
            }
        }

        let st = app.state::<IapState>();
        let mut s = st.inner.lock().unwrap();
        s.running = false;
        s.connected = false;
    });

    Ok(format!("TCP Server 已启动在 :{}", TCP_PORT))
}

async fn wait_for_ack(
    stream: &mut TcpStream,
    recv_buf: &mut Vec<u8>,
    expected_resp_cmd: u8,
    timeout_ms: u64,
) -> Result<u8, String> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let mut tmp = [0u8; 256];

    while Instant::now() < deadline {
        while let Some((consumed, resp_cmd, status)) = protocol::find_ack(recv_buf) {
            if resp_cmd == expected_resp_cmd
                || resp_cmd == (expected_resp_cmd & 0x7F | 0x80)
            {
                recv_buf.drain(..consumed);
                if status == 0 {
                    return Ok(resp_cmd & 0x7F);
                } else {
                    return Err(format!("设备返回错误 status=0x{:02X}", status));
                }
            }
            recv_buf.drain(..consumed);
        }

        match tokio::time::timeout(Duration::from_millis(200), stream.read(&mut tmp)).await {
            Ok(Ok(0)) => return Err("连接已断开".into()),
            Ok(Ok(n)) => recv_buf.extend_from_slice(&tmp[..n]),
            Ok(Err(e)) => return Err(format!("读取错误: {}", e)),
            Err(_) => {}
        }
    }

    Err("等待ACK超时".into())
}

#[tauri::command]
pub async fn stop_tcp_server(app_handle: AppHandle) -> Result<String, String> {
    let state = app_handle.state::<IapState>();
    let mut s = state.inner.lock().unwrap();
    if let Some(tx) = s.shutdown_tx.take() {
        let _ = tx.send(());
        s.running = false;
        s.connected = false;
        Ok("服务器已停止".into())
    } else {
        Err("服务器未在运行".into())
    }
}

#[tauri::command]
pub async fn get_server_status(app_handle: AppHandle) -> Result<(bool, bool), String> {
    let state = app_handle.state::<IapState>();
    let s = state.inner.lock().unwrap();
    Ok((s.running, s.connected))
}

pub(crate) async fn send_frame_and_wait(
    app_handle: &AppHandle,
    frame: Vec<u8>,
    expected_resp_cmd: u8,
) -> Result<u8, String> {
    let state = app_handle.state::<IapState>();
    let cmd_tx = {
        let s = state.inner.lock().unwrap();
        if !s.connected {
            return Err("设备未连接".into());
        }
        s.cmd_tx.clone().ok_or_else(|| "服务器未运行".to_string())?
    };

    let (resp_tx, resp_rx) = oneshot::channel();
    cmd_tx
        .send(ServerCmd::SendAndWaitAck {
            frame,
            expected_resp_cmd,
            resp_tx,
        })
        .await
        .map_err(|_| "服务器已停止".to_string())?;

    resp_rx.await.map_err(|_| "服务器已停止".to_string())?
}

#[tauri::command]
pub async fn send_device_config(
    app_handle: AppHandle,
    config: crate::config::DeviceConfig,
) -> Result<String, String> {
    let data = protocol::pack_device_identity(&config);
    let frame = protocol::build_frame(protocol::CMD_CFG_DEVICE_ID, &data);
    send_frame_and_wait(&app_handle, frame, protocol::CMD_CFG_DEVICE_ID | 0x80).await?;
    Ok("设备身份配置已发送，设备返回成功".into())
}

#[tauri::command]
pub async fn send_network_config(
    app_handle: AppHandle,
    config: crate::config::NetworkConfig,
) -> Result<String, String> {
    let data = protocol::pack_network_config(&config);
    let frame = protocol::build_frame(protocol::CMD_CFG_NETWORK, &data);
    send_frame_and_wait(&app_handle, frame, protocol::CMD_CFG_NETWORK | 0x80).await?;
    Ok("网络配置已发送，设备返回成功".into())
}

#[tauri::command]
pub async fn start_iap_upgrade(
    app_handle: AppHandle,
    firmware_path: String,
) -> Result<String, String> {
    let firmware =
        std::fs::read(&firmware_path).map_err(|e| format!("读取固件失败: {}", e))?;

    let state = app_handle.state::<IapState>();
    let cmd_tx = {
        let s = state.inner.lock().unwrap();
        if !s.connected {
            return Err("设备未连接".into());
        }
        s.cmd_tx.clone().ok_or_else(|| "服务器未运行".to_string())?
    };

    emit_log(&app_handle, "发送IAP升级命令...");
    let frame = protocol::build_frame(protocol::CMD_IAP_317, &[]);
    let (resp_tx, resp_rx) = oneshot::channel();
    cmd_tx
        .send(ServerCmd::SendAndWaitAck {
            frame,
            expected_resp_cmd: protocol::CMD_IAP_317 | 0x80,
            resp_tx,
        })
        .await
        .map_err(|_| "服务器已停止".to_string())?;
    resp_rx.await.map_err(|_| "服务器已停止".to_string())??;

    emit_log(&app_handle, "317已确认，开始发送固件...");

    let fw_len = firmware.len();
    let (resp_tx, resp_rx) = oneshot::channel();
    cmd_tx
        .send(ServerCmd::SendRaw {
            data: firmware,
            resp_tx,
        })
        .await
        .map_err(|_| "服务器已停止".to_string())?;
    resp_rx.await.map_err(|_| "服务器已停止".to_string())??;

    emit_log(&app_handle, &format!("固件发送完成 ({} 字节)", fw_len));
    emit_event(&app_handle, "iap-complete", "");

    Ok("固件已发送，等待设备校验并重启...".into())
}

#[tauri::command]
pub async fn start_ota_upgrade(
    app_handle: AppHandle,
    board: String,
    firmware_path: String,
) -> Result<String, String> {
    let cmd = match board.as_str() {
        "front" => protocol::CMD_OTA_FRONT,
        "back" => protocol::CMD_OTA_BACK,
        _ => return Err("board 必须是 'front' 或 'back'".into()),
    };

    let fota_data =
        std::fs::read(&firmware_path).map_err(|e| format!("读取fota文件失败: {}", e))?;

    let packets = protocol::split_fota_packets(&fota_data)?;

    let state = app_handle.state::<IapState>();
    let cmd_tx = {
        let s = state.inner.lock().unwrap();
        if !s.connected {
            return Err("设备未连接".into());
        }
        s.cmd_tx.clone().ok_or_else(|| "服务器未运行".to_string())?
    };

    emit_log(&app_handle, &format!("开始OTA {} 板 ({} 包)", board, packets.len()));

    for (i, packet) in packets.iter().enumerate() {
        let frame = protocol::build_frame(cmd, packet);
        let (resp_tx, resp_rx) = oneshot::channel();
        cmd_tx
            .send(ServerCmd::SendAndWaitAck {
                frame,
                expected_resp_cmd: cmd | 0x80,
                resp_tx,
            })
            .await
            .map_err(|_| "服务器已停止".to_string())?;
        let status = resp_rx.await.map_err(|_| "服务器已停止".to_string())??;
        emit_log(&app_handle, &format!("  包 {}/{} 完成, status=0x{:02X}", i + 1, packets.len(), status));
    }

    emit_log(&app_handle, "OTA升级完成!");
    emit_event(&app_handle, "ota-complete", &board);

    Ok(format!("OTA {} 板升级完成 ({} 包)", board, packets.len()))
}
