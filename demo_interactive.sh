#!/bin/bash

echo "🎮 omobaf 互動式 CLI 演示"
echo "=========================="
echo ""

# 啟動後端
echo "📡 啟動 omobab 後端..."
cd /mnt/d/Nobu/open_moba_backend
cargo run --bin omobab > /tmp/omobab.log 2>&1 &
BACKEND_PID=$!
echo "後端 PID: $BACKEND_PID"

sleep 5
cd /mnt/d/Nobu/omobaf

echo ""
echo "🚀 啟動互動式客戶端..."
echo ""
echo "📋 將執行以下命令序列："
echo "  1. help         - 查看幫助"
echo "  2. abilities    - 查看技能"
echo "  3. config       - 查看配置"
echo "  4. connect localhost - 連接服務器"
echo "  5. status       - 查看狀態"
echo "  6. play         - 開始遊戲"
echo "  7. move 100 200 - 移動角色"
echo "  8. cast sniper_mode 150 250 1 - 施放技能"
echo "  9. attack 200 300 - 攻擊"
echo "  10. status      - 再次查看狀態"
echo "  11. exit        - 退出"
echo ""

# 創建命令文件
cat > /tmp/omobaf_commands.txt << EOF
help
abilities
config
connect localhost
status
play
move 100 200
cast sniper_mode 150 250 1
attack 200 300
status
exit
EOF

# 執行互動式客戶端
timeout 30s ./target/release/omobaf < /tmp/omobaf_commands.txt

# 清理
kill $BACKEND_PID 2>/dev/null
pkill -f omobab 2>/dev/null
rm -f /tmp/omobaf_commands.txt

echo ""
echo "✅ 互動式 CLI 演示完成！"
echo ""
echo "💡 使用方法："
echo "   直接運行 ./target/release/omobaf 即可進入互動式模式"
echo "   或使用命令參數執行單個操作："
echo "   ./target/release/omobaf --server-ip localhost connect"