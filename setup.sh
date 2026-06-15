#!/bin/bash
# CH32V317 配置工具 - 环境安装脚本
# 使用方法: bash setup.sh

set -e

echo "==================================="
echo " CH32V317 配置工具 - 环境安装"
echo "==================================="

# 1. Install system dependencies
echo ""
echo "[1/4] 安装系统依赖..."
if command -v apt-get &>/dev/null; then
    echo "检测到 apt，正在安装依赖..."
    echo "如果提示输入密码，请输入你的 sudo 密码"
    sudo apt-get update
    sudo apt-get install -y \
        libwebkit2gtk-4.0-dev \
        libsoup2.4-dev \
        libgtk-3-dev \
        libappindicator3-dev \
        librsvg2-dev \
        pkg-config \
        build-essential
elif command -v dnf &>/dev/null; then
    sudo dnf install -y webkit2gtk3-devel libsoup-devel gtk3-devel libappindicator-gtk3-devel
fi

# 2. Ensure Rust is installed
echo ""
echo "[2/4] 检查 Rust 环境..."
if ! command -v rustc &>/dev/null; then
    echo "安装 Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust: $(rustc --version)"
fi

# 3. Install frontend dependencies
echo ""
echo "[3/4] 安装前端依赖..."
if command -v pnpm &>/dev/null; then
    pnpm install --shamefully-hoist=true
elif command -v npm &>/dev/null; then
    npm install --legacy-peer-deps
else
    echo "错误: 请先安装 Node.js"
    exit 1
fi

# 4. Done
echo ""
echo "[4/4] 安装完成!"
echo ""
echo "启动开发模式: cargo tauri dev"
echo "构建发布包:   cargo tauri build"
echo "单独运行前端: pnpm dev  (无 Rust 后端功能)"
echo ""
echo "目录结构:"
echo "  src/               - Vue3 前端代码"
echo "  src/components/    - UI 组件"
echo "  src/stores/        - Pinia 状态管理"
echo "  src-tauri/src/     - Rust 后端代码"
echo "  resources/         - 固件模板文件"
echo "==================================="
