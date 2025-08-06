# omobaf 前端開發狀態 - 2025/08/06

## 專案概述
omobaf (Open MOBA Frontend) - 假遊戲前端客戶端，用於測試 omobab 後端的遊戲邏輯。

## ✅ 已完成功能

### 1. 前端自動連接本地端 (已完成)
- **問題**: 前端執行時沒有預設直連本地端
- **解決方案**: 在 `InteractiveCli::run()` 中添加 `auto_connect_localhost()` 方法
- **文件**: `src/interactive.rs:41-49`
- **狀態**: ✅ 前端啟動時自動連接到 127.0.0.1:1883

### 2. 前端主動請求當前遊戲畫面狀態 (已完成)
- **問題**: 前端沒有主動向後端請求遊戲畫面狀態
- **解決方案**: 實作畫面狀態請求循環，每3秒發送一次MQTT請求
- **文件**: `src/game_client.rs:300-350`
- **協議**: 
  - 請求主題: `td/{player_name}/request`
  - 回應主題: `td/{player_name}/screen_response`

### 3. 增強畫面請求協議 (已完成)
- **功能**: 支援兩種請求模式
  - **player_centered**: 以玩家為中心 ±60x, ±40y 範圍
  - **fixed_area**: 固定範圍 (如 -40,-40 到 40,40)
- **數據結構**: 完整的畫面數據(實體、玩家、投射物、地形)
- **文件**: `src/mqtt_handler.rs:376-429`

### 4. MQTT處理器增強 (已完成)
- **新增**: `handle_screen_response_message()` 方法
- **功能**: 處理 `td/+/screen_response` 主題
- **整合**: 將網路實體轉換為本地實體格式
- **文件**: `src/mqtt_handler.rs:242-305`

### 5. 前端自動啟動後端功能 (已完成) 🎉
- **核心功能**: 前端根據配置自動啟動和管理後端程序
- **配置系統**: `config.toml` 配置後端執行路徑和參數
- **後端管理**: 支援 start/stop/restart/status 命令
- **工作目錄**: 正確設定後端工作目錄以讀取 log4rs.yml

#### 實作檔案:
- `src/config.rs` - 配置檔案處理
- `src/backend_manager.rs` - 後端程序生命週期管理
- `src/interactive.rs` - 整合後端管理到交互界面
- `config.toml` - 配置檔案

#### 後端管理命令:
```bash
backend start    # 啟動後端
backend stop     # 停止後端
backend restart  # 重啟後端
backend status   # 查看狀態
```

### 6. 命令列直接進入視圖模式 (已完成)
- **命令**: `cargo run --bin omobaf -- interactive --auto-view`
- **功能**: 直接進入遊戲並啟動實時視圖
- **參數**: `--size`, `--show-vision`
- **文件**: `src/cli.rs:27-34`

## 🔧 配置文件

### config.toml
```toml
[server]
mqtt_host = "127.0.0.1"
mqtt_port = 1883

[backend]
executable_path = "../open_moba_backend/target/debug/omobab"
args = []
working_directory = "../open_moba_backend"

[backend.env]
RUST_LOG = "info"

[frontend]
player_name = "TestPlayer"
hero_type = "saika_magoichi"
auto_start_backend = true
backend_start_delay = 1000
backend_shutdown_timeout = 5000
```

## 📁 專案結構

```
omobaf/
├── src/
│   ├── main.rs                  # 主程序入口
│   ├── cli.rs                   # 命令列參數處理
│   ├── interactive.rs           # 互動式CLI (含後端管理)
│   ├── game_client.rs           # 遊戲客戶端和MQTT通信
│   ├── mqtt_handler.rs          # MQTT訊息處理器
│   ├── game_state.rs            # 遊戲狀態管理
│   ├── player.rs                # 玩家模擬器
│   ├── terminal_view.rs         # 終端視圖渲染
│   ├── config.rs                # 配置檔案處理
│   └── backend_manager.rs       # 後端程序管理
├── config.toml                  # 配置檔案
├── test_backend_auto_start.sh   # 後端自動啟動測試腳本
└── Cargo.toml                   # Rust專案配置
```

## 🚀 使用方式

### 基本啟動 (推薦)
```bash
# 啟動前端，自動啟動後端並連接
./target/debug/omobaf
```

### 直接進入視圖模式
```bash
# 直接進入遊戲實時視圖
./target/debug/omobaf interactive --auto-view --size 20
```

### 後端管理
```bash
# 在互動模式中
backend status   # 查看後端狀態
backend restart  # 重啟後端
backend stop     # 停止後端
```

## 🔄 工作流程

1. **啟動**: `./target/debug/omobaf`
2. **自動啟動後端**: 根據 `config.toml` 設定
3. **自動連接**: 連接到後端 MQTT 服務器 (127.0.0.1:1883)
4. **互動模式**: 支援遊戲命令和後端管理
5. **退出清理**: 自動停止管理的後端程序

## 🌐 MQTT 通信協議

### 畫面狀態請求
```json
{
  "t": "screen_request",
  "a": "get_screen_area",
  "d": {
    "player_name": "TestPlayer",
    "request_type": "player_centered",
    "center_x": 0.0,
    "center_y": 0.0,
    "width": 120.0,
    "height": 80.0,
    "timestamp": 1672531200000
  }
}
```

### 畫面狀態回應
```json
{
  "t": "screen_response",
  "d": {
    "area": {"min_x": -60, "min_y": -40, "max_x": 60, "max_y": 40},
    "entities": [...],
    "players": [...],
    "projectiles": [...],
    "terrain": [...],
    "timestamp": 1672531200001
  }
}
```

## ✨ 主要特性

- **零配置啟動**: 只需執行前端，後端自動啟動
- **完整的後端管理**: 啟動、停止、重啟、狀態查詢
- **自動連接**: 前端自動連接到後端 MQTT 服務器
- **實時畫面更新**: 每3秒自動請求遊戲狀態
- **靈活的視圖模式**: 支援固定範圍和玩家中心視圖
- **終端視圖**: 支援滑鼠操作的實時遊戲視圖
- **完整的MQTT通信**: 支援所有遊戲相關的MQTT訊息

## 🧪 測試

### 後端自動啟動測試
```bash
./test_backend_auto_start.sh
```

### 畫面請求協議測試  
```bash
./test_screen_request.sh
```

## 🎯 用戶需求達成狀態

✅ **完全達成**: "前端要自己開後端來測，用設定檔設定後端執行路徑，這樣我們只需要啟動前端就好，後端給前端自己控制才對"

- ✅ 前端自動啟動後端
- ✅ 配置檔案設定後端路徑
- ✅ 只需啟動前端即可
- ✅ 前端完全控制後端生命週期

## 📊 開發狀態

| 任務 | 狀態 | 完成度 |
|------|------|--------|
| 前端自動連接本地端 | ✅ 完成 | 100% |
| 畫面狀態請求機制 | ✅ 完成 | 100% |
| 畫面請求協議增強 | ✅ 完成 | 100% |
| MQTT處理器支援 | ✅ 完成 | 100% |
| 請求模式實作 | ✅ 完成 | 100% |
| 前端自動啟動後端 | ✅ 完成 | 100% |
| 配置檔案系統 | ✅ 完成 | 100% |
| 後端程序管理 | ✅ 完成 | 100% |

**總體完成度: 100%** 🎉

---

*最後更新: 2025/08/06 18:05*  
*開發者: Claude + 用戶協作開發*