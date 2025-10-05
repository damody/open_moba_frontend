# omobaf 源碼架構

本目錄包含 omobaf 前端的所有源碼模組。每個模組都有其特定功能和職責。

## 📁 模組結構

### 核心模組

- **`main.rs`** - 程式入口點
  - 初始化配置和日誌系統
  - 處理命令行參數
  - 啟動後端管理器（如果配置啟用）
  - 協調各模組運行

- **`game_client.rs`** - 遊戲客戶端核心
  - 管理整個遊戲會話生命週期
  - 整合 MQTT、遊戲狀態、終端視圖
  - 處理玩家操作和服務器響應
  - 坐標系統：2D 坐標 (x, y)，地圖大小 800x600

- **`backend_manager.rs`** - 後端程序管理器
  - 自動啟動/停止 omobab 後端
  - 將後端輸出重定向到 `backend.log`
  - 清理舊進程（Windows: taskkill，Unix: pkill）
  - 程序退出時自動清理

### 網絡通信

- **`mqtt_handler.rs`** - MQTT 訊息處理器
  - 管理 MQTT 連接和重連
  - 訂閱主題：
    - `td/+/send` - 遊戲狀態更新
    - `ability_test/response` - 能力測試響應
  - 發布主題：
    - `td/{player_name}/action` - 玩家操作
  - 訊息格式：JSON

### 遊戲邏輯

- **`game_state.rs`** - 遊戲狀態管理
  - 維護本地遊戲狀態
  - 追蹤玩家、召喚物、其他實體
  - 狀態同步和驗證
  - 冷卻時間管理

- **`player.rs`** - 玩家和英雄定義
  - 英雄類型定義（雜賀孫一、伊達政宗）
  - 能力定義和屬性
  - 玩家狀態結構

### 用戶介面

- **`cli.rs`** - 命令行介面
  - 使用 clap 4.0 定義命令結構
  - 支援子命令：play、move、cast、attack、status、auto、demo
  - 參數解析和驗證

- **`terminal_logger.rs`** - 日誌系統
  - 自定義日誌格式化
  - 支援不同日誌級別
  - 彩色輸出（如果終端支援）

### 配置管理

- **`config.rs`** - 配置系統
  - 讀取和解析 `config.toml`
  - 配置結構定義
  - 預設值和驗證

## 📂 子模組

### terminal_view/ - 終端視圖系統
完整的終端遊戲視圖實現。[詳細文檔](./terminal_view/README.md)

- `mod.rs` - 模組入口和公共介面
- `viewport.rs` - 視口管理和坐標轉換
- `renderer.rs` - 渲染邏輯
- `display.rs` - 終端顯示管理
- `input.rs` - 鍵盤輸入處理

### interactive/ - 互動模式系統
提供即時互動遊戲體驗。[詳細文檔](./interactive/README.md)

- `mod.rs` - 模組入口
- `session.rs` - 會話管理
- `commands.rs` - 命令處理

## 🔄 資料流

```
main.rs 
  ├─> backend_manager.rs (自動啟動後端)
  ├─> config.rs (載入配置)
  └─> game_client.rs
       ├─> mqtt_handler.rs (網絡通信)
       ├─> game_state.rs (狀態管理)
       ├─> terminal_view/ (視覺呈現)
       └─> interactive/ (用戶互動)
```

## 🛠️ 開發指南

### 添加新功能

1. **新增英雄**：修改 `player.rs`，添加英雄類型和能力定義
2. **新增命令**：修改 `cli.rs` 和 `interactive/commands.rs`
3. **新增視圖元素**：修改 `terminal_view/renderer.rs`
4. **新增訊息類型**：修改 `mqtt_handler.rs` 和 `game_state.rs`

### 調試建議

- 使用 `RUST_LOG=debug` 環境變數查看詳細日誌
- 檢查 `backend.log` 查看後端輸出
- 使用 mosquitto_sub 監控 MQTT 訊息流

## 📝 編碼規範

- 使用 Rust 2021 Edition
- 遵循 Rust 官方編碼風格
- 所有公共 API 需要文檔註釋
- 錯誤處理使用 `anyhow::Result`
- 異步操作使用 Tokio runtime