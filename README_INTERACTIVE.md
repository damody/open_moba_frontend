# omobaf 互動式 CLI 介面

## 🎮 功能特點

omobaf 現在提供了一個完整的互動式 CLI 介面，讓你可以在執行期間即時輸入指令與遊戲後端互動。

## 🚀 啟動方式

### 互動式模式（推薦）
直接運行程式，不帶任何參數：
```bash
./target/release/omobaf
```

### 命令行模式
使用參數執行單個命令：
```bash
./target/release/omobaf --server-ip localhost connect
./target/release/omobaf abilities
```

## 📋 互動式指令列表

在互動式模式中，你可以使用以下指令：

| 指令 | 說明 | 範例 |
|------|------|------|
| `help` 或 `?` | 顯示幫助訊息 | `help` |
| `connect <ip> [port]` | 連接到遊戲服務器 | `connect localhost 1883` |
| `disconnect` | 斷開連接 | `disconnect` |
| `config [key] [value]` | 查看或修改配置 | `config name Player1` |
| `status` | 查看當前狀態 | `status` |
| `play [hero]` | 開始遊戲 | `play saika_magoichi` |
| `move <x> <y>` | 移動到指定位置 | `move 100 200` |
| `cast <ability> [x] [y] [level]` | 施放技能 | `cast sniper_mode 150 250 1` |
| `attack <x> <y>` | 攻擊指定位置 | `attack 200 300` |
| `abilities` | 列出可用技能 | `abilities` |
| `auto [duration]` | 自動遊戲模式 | `auto 30` |
| `clear` | 清除畫面 | `clear` |
| `exit` 或 `quit` | 退出程式 | `exit` |

## 🎯 使用範例

### 基本遊戲流程
```
[未連接] > connect localhost
→ 連接到 localhost:1883...
✓ 連接成功！

[已連接] > play
→ 開始遊戲，英雄: saika_magoichi
✓ 已進入遊戲！

[遊戲中] > move 100 200
→ 移動到 (100, 200)
✓ 移動完成

[遊戲中] > cast sniper_mode 150 250 1
→ 施放技能: sniper_mode
✓ 技能施放成功

[遊戲中] > status
遊戲狀態:
----------------------------------------
  連接狀態: InGame
  玩家: TestPlayer
  英雄: saika_magoichi
  位置: (100.0, 200.0)
  生命值: 1000/1000

[遊戲中] > exit
→ 斷開連接...
👋 再見！
```

### 配置管理
```
[未連接] > config
當前配置:
  服務器: 45.32.32.40:1883
  客戶端ID: omobaf_player
  玩家名稱: TestPlayer
  英雄類型: saika_magoichi

[未連接] > config name MyHero
✓ 玩家名稱設為: MyHero

[未連接] > config hero date_masamune
✓ 英雄類型設為: date_masamune
```

## 🌈 視覺特色

- **彩色輸出**：使用不同顏色區分狀態、成功、錯誤等訊息
- **狀態指示器**：提示符會顯示當前連接狀態
  - `[未連接]` - 紅色
  - `[已連接]` - 綠色
  - `[遊戲中]` - 亮綠色
  - `[連接中]` - 黃色
  - `[錯誤]` - 亮紅色

## 🔧 技術特點

1. **非阻塞式輸入**：使用 Tokio 異步運行時
2. **即時狀態更新**：連接狀態實時反映在提示符
3. **錯誤處理**：友好的錯誤訊息提示
4. **命令歷史**：支持標準終端的命令歷史功能
5. **優雅退出**：退出時自動斷開連接

## 📡 MQTT 整合

互動式 CLI 完全整合了 MQTT 通信：
- 所有遊戲指令都會通過 MQTT 發送到後端
- 接收並處理來自後端的遊戲狀態更新
- 支持 `td/all/res` 和 `td/+/send` 主題

## 🚦 快速開始

1. 確保 omobab 後端正在運行
2. 執行 `./target/release/omobaf`
3. 輸入 `help` 查看所有可用指令
4. 使用 `connect localhost` 連接到本地後端
5. 開始遊戲並享受互動體驗！