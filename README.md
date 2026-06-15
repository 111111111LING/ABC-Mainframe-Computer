# CH32V317 设备配置工具

基于 **Tauri v1 + Vue3 + Rust** 的桌面端配置与 IAP 升级工具。

## 功能

- **设备身份配置**: ProductID, DeviceName, SecKey
- **网络配置**: MAC 地址, DHCP/静态IP, 网关, 子网掩码
- **MQTT 配置**: 服务器 IP, 端口, NTP 服务器
- **固件生成**: 二进制打补丁 + IAP 头生成
- **IAP 升级**: TCP Server 模式, 通过以太网发送固件

## 前置要求

- Node.js >= 18
- Rust (已安装)
- 系统库: `libwebkit2gtk-4.0-dev`, `libsoup2.4-dev`, `libgtk-3-dev`

## 安装与运行

```bash
# 1. 安装系统依赖 (Ubuntu/Debian)
sudo apt-get install -y libwebkit2gtk-4.0-dev libsoup2.4-dev \
  libgtk-3-dev libappindicator3-dev librsvg2-dev pkg-config

# 2. 安装前端依赖
pnpm install --shamefully-hoist=true

# 3. 开发模式运行
cargo tauri dev

# 4. 构建发布包
cargo tauri build
```

## 项目结构

```
├── src/                    # Vue3 前端
│   ├── components/
│   │   ├── DeviceIdentity.vue   # 设备身份配置
│   │   ├── NetworkConfig.vue    # 网络配置
│   │   ├── MqttConfig.vue       # MQTT 配置
│   │   └── IapFlasher.vue       # IAP 升级界面
│   ├── stores/configStore.js    # Pinia 状态管理
│   └── App.vue                  # 主页面
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── main.rs         # 入口, Tauri commands 注册
│       ├── config.rs       # 配置数据模型
│       ├── firmware.rs     # 二进制打补丁 + IAP 头生成
│       └── iap_server.rs   # TCP IAP 服务器
├── resources/              # 固件模板
├── package.json
├── setup.sh                # 自动安装脚本
└── vite.config.js
```

## 二进制打补丁偏移

| 变量 | 偏移 | 大小 | 说明 |
|------|------|------|------|
| `DESIP` | 0x023ca0 | 4B | MQTT 服务器 IP |
| `GWIPAddr` | 0x023ca4 | 4B | 网关 |
| `IPAddr` | 0x023ca8 | 4B | 设备静态 IP |
| `IPMask` | 0x023cac | 4B | 子网掩码 |
| `NTP_ServerIP` | 0x023cb0 | 4B | NTP 服务器 |
| `desport` | 0x023cb6 | 2B | MQTT 端口 (LE) |
