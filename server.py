#!/usr/bin/env python3
"""CH32V317 配置工具 — Python 全栈服务器
集成了 HTTP 前端服务和 IAP TCP Server。

用法:
  python3 server.py
  然后浏览器打开 http://localhost:8080

  或者指定端口:
  python3 server.py --port 8080
"""
import http.server
import json
import os
import socket
import struct
import sys
import threading
import time
import urllib.parse
from pathlib import Path

IAP_PORT = 1000
DIST_DIR = Path(__file__).parent / "dist"

# ── 配置默认值 ──────────────────────────────────
DEFAULT_CONFIG = {
    "deviceConfig": {
        "productID": "7KdEHCyUDG",
        "deviceName": "DVDR260512010901",
        "secKey": "N0Y0NzY2NDYyQzI2MkU0MjAwRjZCNTEwQkRCMkI4MkU=",
        "bind": 1,
    },
    "networkConfig": {
        "lanDHCP": True,
        "lanIP": "192.168.124.100",
        "lanGateway": "192.168.124.1",
        "lanMask": "255.255.255.0",
        "macAddr": "04:2B:58:09:D2:F3",
    },
    "mqttConfig": {
        "serverIp": "101.132.160.111",
        "serverPort": 8883,
        "ntpServer": "106.14.18.202",
        "ntpPort": 12123,
    },
    "iapConfig": {
        "listenPort": 1000,
        "firmwarePath": "",
    },
}

# ── IAP TCP Server ─────────────────────────────
iap_server = None
iap_thread = None
iap_running = False
iap_log = []
iap_progress = 0
iap_connected = False


def iap_server_thread(port, firmware_data):
    global iap_running, iap_progress, iap_connected, iap_log

    iap_log = []
    iap_progress = 0
    iap_connected = False

    def log(msg):
        ts = time.strftime("%H:%M:%S")
        iap_log.append(f"[{ts}] {msg}")

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    try:
        server.bind(("0.0.0.0", port))
        server.listen(1)
        server.settimeout(1.0)
    except Exception as e:
        log(f"ERROR: 绑定端口 {port} 失败: {e}")
        iap_running = False
        return

    log(f"TCP Server 已启动在 :{port}")
    iap_running = True

    while iap_running:
        try:
            conn, addr = server.accept()
            iap_connected = True
            log(f"设备已连接: {addr[0]}:{addr[1]}")
        except socket.timeout:
            continue
        except Exception as e:
            if iap_running:
                log(f"ERROR: 接受连接失败: {e}")
            break

        total = len(firmware_data)
        sent = 0
        chunk_size = 1460

        while sent < total and iap_running:
            chunk = firmware_data[sent:sent + chunk_size]
            try:
                conn.sendall(chunk)
            except Exception as e:
                log(f"ERROR: 发送失败: {e}")
                break
            sent += len(chunk)
            iap_progress = sent * 100 // total

        if sent == total:
            log("固件传输完成!")
            iap_progress = 100

        conn.close()
        iap_connected = False

    server.close()
    log("TCP Server 已停止")
    iap_running = False


# ── 配置打补丁 ─────────────────────────────────
def patch_firmware(fw_data, config):
    """二进制打补丁，同 browserGenerateFirmware 逻辑"""
    data = bytearray(fw_data)
    nc = config.get("networkConfig", {})

    def patch_ip(offset, ip_str):
        parts = [int(p) for p in ip_str.split(".")]
        for i, v in enumerate(parts):
            data[offset + i] = v

    patch_ip(0x023CA0, config.get("mqttConfig", {}).get("serverIp", "0.0.0.0"))
    patch_ip(0x023CA4, nc.get("lanMask", "255.255.255.0"))
    patch_ip(0x023CA8, nc.get("lanIP", "0.0.0.0"))
    patch_ip(0x023CAC, nc.get("lanGateway", "0.0.0.0"))
    patch_ip(0x023CB0, config.get("mqttConfig", {}).get("ntpServer", "0.0.0.0"))

    port = config.get("mqttConfig", {}).get("serverPort", 8883)
    data[0x023CB6] = port & 0xFF
    data[0x023CB7] = (port >> 8) & 0xFF

    return bytes(data)


def build_iap_header(firmware_data):
    """生成 512 字节 IAP 头"""
    total_len = 512 + len(firmware_data)
    checksum = 0
    for i in range(0, len(firmware_data), 4):
        val = firmware_data[i]
        if i + 1 < len(firmware_data):
            val |= firmware_data[i + 1] << 8
        if i + 2 < len(firmware_data):
            val |= firmware_data[i + 2] << 16
        if i + 3 < len(firmware_data):
            val |= firmware_data[i + 3] << 24
        checksum = (checksum + val) & 0xFFFFFFFF

    header = bytearray(512)
    header[0:8] = b"WCHNET\0\0"
    struct.pack_into("<I", header, 8, total_len)
    struct.pack_into("<I", header, 12, checksum)
    return bytes(header)


# ── HTTP Server ────────────────────────────────
class APIHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(DIST_DIR), **kwargs)

    def do_OPTIONS(self):
        self.send_cors()
        self.send_response(200)
        self.end_headers()

    def send_cors(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        body = self.rfile.read(int(self.headers.get("Content-Length", 0)))
        self.send_cors()

        if parsed.path == "/api/start-iap":
            self.handle_start_iap(body)
        elif parsed.path == "/api/stop-iap":
            self.handle_stop_iap()
        elif parsed.path == "/api/generate-firmware":
            self.handle_generate_firmware(body)
        else:
            self.send_json({"error": "unknown"}, 404)

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        self.send_cors()

        if parsed.path == "/api/status":
            self.send_json({
                "running": iap_running,
                "connected": iap_connected,
                "progress": iap_progress,
                "logs": iap_log[-50:],  # 最近 50 条
            })
        else:
            super().do_GET()

    def send_json(self, data, code=200):
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps(data, ensure_ascii=False).encode())

    def handle_start_iap(self, body):
        global iap_thread, iap_running
        if iap_running:
            self.send_json({"error": "服务器已在运行"}, 400)
            return

        try:
            data = json.loads(body)
            port = data.get("port", IAP_PORT)
            firmware_b64 = data.get("firmware")
            if not firmware_b64:
                self.send_json({"error": "缺少固件数据"}, 400)
                return

            import base64
            firmware_data = base64.b64decode(firmware_b64)
        except Exception as e:
            self.send_json({"error": f"参数错误: {e}"}, 400)
            return

        iap_thread = threading.Thread(
            target=iap_server_thread, args=(port, firmware_data), daemon=True
        )
        iap_thread.start()
        self.send_json({"status": "started", "port": port})

    def handle_stop_iap(self):
        global iap_running
        if not iap_running:
            self.send_json({"error": "服务器未运行"}, 400)
            return
        iap_running = False
        self.send_json({"status": "stopped"})

    def handle_generate_firmware(self, body):
        """接收 DNS.bin + 配置 → 返回 DNS.bin.bin（base64）"""
        try:
            data = json.loads(body)
            fw_b64 = data.get("firmware")
            config = data.get("config", {})
            if not fw_b64:
                self.send_json({"error": "缺少固件数据"}, 400)
                return

            import base64
            fw_data = base64.b64decode(fw_b64)
        except Exception as e:
            self.send_json({"error": f"参数错误: {e}"}, 400)
            return

        patched = patch_firmware(fw_data, config)
        header = build_iap_header(patched)
        output = header + patched

        checksum = 0
        for i in range(0, len(patched), 4):
            val = patched[i]
            if i + 1 < len(patched):
                val |= patched[i + 1] << 8
            if i + 2 < len(patched):
                val |= patched[i + 2] << 16
            if i + 3 < len(patched):
                val |= patched[i + 3] << 24
            checksum = (checksum + val) & 0xFFFFFFFF

        self.send_json({
            "firmware": base64.b64encode(output).decode(),
            "size": len(output),
            "checksum": f"0x{checksum:08X}",
        })

    def log_message(self, format, *args):
        pass  # 不打印请求日志


# ── 入口 ───────────────────────────────────────
def main():
    import argparse
    parser = argparse.ArgumentParser(description="CH32V317 配置工具服务器")
    parser.add_argument("--port", type=int, default=8080, help="HTTP 端口 (默认 8080)")
    parser.add_argument("--host", default="0.0.0.0", help="监听地址 (默认 0.0.0.0)")
    args = parser.parse_args()

    if not DIST_DIR.exists():
        print(f"错误: 找不到前端构建目录 {DIST_DIR}")
        print("请先运行: npm run build")
        sys.exit(1)

    server = http.server.HTTPServer((args.host, args.port), APIHandler)
    print(f"╔══════════════════════════════════════╗")
    print(f"║  CH32V317 配置工具服务器已启动        ║")
    print(f"║                                      ║")
    print(f"║  浏览器打开: http://localhost:{args.port}  ║")
    print(f"║  IAP TCP 端口: {IAP_PORT}                      ║")
    print(f"╚══════════════════════════════════════╝")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n正在停止...")
        global iap_running
        iap_running = False
        server.shutdown()


if __name__ == "__main__":
    main()
