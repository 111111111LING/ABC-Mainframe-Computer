#!/usr/bin/env python3
"""IAP 固件发送工具 — TCP Server 模式
板子 (ETH_IAP Bootloader) 会连 PC:1000，此脚本把固件发过去。

用法:
  python3 iap_send.py DNS.bin.bin
  python3 iap_send.py DNS.bin.bin --port 1000
"""
import socket
import sys
import argparse
import os

def main():
    parser = argparse.ArgumentParser(description="IAP 固件发送工具")
    parser.add_argument("firmware", help="固件文件路径 (DNS.bin.bin)")
    parser.add_argument("--port", type=int, default=1000, help="监听端口 (默认 1000)")
    args = parser.parse_args()

    if not os.path.exists(args.firmware):
        print(f"错误: 文件不存在 {args.firmware}")
        sys.exit(1)

    fw_size = os.path.getsize(args.firmware)
    print(f"固件: {args.firmware} ({fw_size} 字节)")
    print(f"监听: 0.0.0.0:{args.port}")
    print("等待板子连接...")

    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("0.0.0.0", args.port))
    server.listen(1)

    conn, addr = server.accept()
    print(f"设备已连接: {addr[0]}:{addr[1]}")

    with open(args.firmware, "rb") as f:
        data = f.read()

    total = len(data)
    sent = 0
    chunk_size = 1460

    while sent < total:
        chunk = data[sent:sent + chunk_size]
        conn.sendall(chunk)
        sent += len(chunk)
        pct = sent * 100 // total
        print(f"\r发送中: {sent}/{total} ({pct}%)", end="", flush=True)

    print(f"\n发送完成! 共 {total} 字节")
    conn.close()
    server.close()

if __name__ == "__main__":
    main()
