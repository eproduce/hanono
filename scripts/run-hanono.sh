#!/bin/bash
# Hanono 离线部署启动脚本
# 适用: 麒麟 V10 / 无 FUSE / 无网络 环境
# 用法: ./run-hanono.sh [AppImage路径]

set -e

APPIMAGE="${1:-Hanono_*.AppImage}"
APPIMAGE=$(ls $APPIMAGE 2>/dev/null | head -1)

if [ -z "$APPIMAGE" ]; then
    echo "❌ 未找到 AppImage 文件，请指定路径: ./run-hanono.sh ./Hanono_0.4.0_amd64.AppImage"
    exit 1
fi

echo "🎵 Hanono - 沉浸式音乐播放器"
echo "   AppImage: $APPIMAGE"

EXTRACT_DIR="./hanono-app"

# 首次运行解压（只需一次）
if [ ! -f "$EXTRACT_DIR/AppRun" ]; then
    echo "📦 首次运行，正在解压 AppImage（无需 root/FUSE）..."
    chmod +x "$APPIMAGE"
    "$APPIMAGE" --appimage-extract > /dev/null 2>&1
    mv squashfs-root "$EXTRACT_DIR"
    echo "✅ 解压完成: $EXTRACT_DIR"
fi

# 启动
echo "▶️  启动 Hanono..."
"$EXTRACT_DIR/AppRun" &

echo "🎶 Hanono 已启动！(PID: $!)"
echo "   下次直接运行: ./$EXTRACT_DIR/AppRun"
