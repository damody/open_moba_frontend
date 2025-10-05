# omobaf - Open MOBA Frontend

假遊戲前端客戶端，用於測試 `omobab` (Open MOBA Backend) 後端的遊戲邏輯。

## 專案概述

`omobaf` 是一個 Rust CLI 工具，模擬真實遊戲客戶端行為，專注於處理 MQTT 遊戲訊息，用來驗證後端遊戲邏輯是否正確運作。

### 主要特色

- 🎮 **真實遊戲模擬**: 模擬真實玩家的遊戲操作行為
- 📡 **MQTT 通信**: 處理與 omobab 後端的 MQTT 訊息通信
- 🤖 **自動遊戲模式**: 支援自動化操作序列測試
- 🎯 **英雄支援**: 支援雜賀孫一和伊達政宗兩個英雄
- 🛠️ **簡潔 CLI**: 直觀的命令行操作介面
- 📊 **狀態同步**: 本地遊戲狀態與服務器同步驗證

## 安裝和建置

### 系統需求

- Rust 1.70+
- 運行中的 `omobab` 後端服務
- MQTT Broker (預設使用 45.32.32.40:1883)

### 建置專案

```bash
cd /mnt/d/Nobu/omobaf
cargo build --release
```

### 運行

```bash
# 使用預設設定
./target/release/omobaf --help

# 或直接用 cargo 運行
cargo run -- --help
```

## 使用方法

### 基本命令

#### 1. 連接到遊戲服務器

```bash
omobaf connect --server-ip 45.32.32.40 --server-port 1883
```

#### 2. 開始遊戲

```bash
# 使用雜賀孫一
omobaf play --hero saika_magoichi --player-name "TestPlayer1"

# 使用伊達政宗
omobaf play --hero date_masamune --player-name "TestPlayer2"
```

#### 3. 遊戲操作

```bash
# 移動
omobaf move 300 200

# 施放技能
omobaf cast sniper_mode --x 350 --y 250 --level 1
omobaf cast saika_reinforcements --x 400 --y 300

# 攻擊
omobaf attack 450 350 --attack-type basic
```

#### 4. 查看狀態

```bash
omobaf status
```

#### 5. 自動遊戲模式

```bash
# 自動遊戲 60 秒
omobaf auto --duration 60
```

#### 6. 執行演示序列

```bash
omobaf demo
```

#### 7. 查看可用技能

```bash
omobaf abilities
```

### 支援的英雄和技能

#### 雜賀孫一 (saika_magoichi)
- `sniper_mode`: 狙擊模式
- `saika_reinforcements`: 雜賀眾（召喚雜賀鐵炮兵）
- `rain_iron_cannon`: 雨鐵炮
- `three_stage_technique`: 三段擊

#### 伊達政宗 (date_masamune)
- `flame_blade`: 火焰刀
- `fire_dash`: 火焰衝刺
- `flame_assault`: 火焰突擊
- `matchlock_gun`: 火繩槍

### 完整使用範例

```bash
# 1. 連接並開始遊戲
omobaf play --hero saika_magoichi --player-name "Player1" \
  --server-ip 45.32.32.40 --server-port 1883

# 2. 移動到戰場中央
omobaf move 400 300

# 3. 召喚雜賀眾
omobaf cast saika_reinforcements --x 450 --y 350

# 4. 使用狙擊模式
omobaf cast sniper_mode

# 5. 攻擊敵人
omobaf attack 500 400 --attack-type ranged

# 6. 查看當前狀態
omobaf status

# 7. 自動遊戲 30 秒
omrobaf auto --duration 30
```

## 📚 文檔導航

詳細的模組文檔請參考以下連結：

- **[源碼架構總覽](./src/README.md)** - 所有模組的詳細說明
- **[終端視圖系統](./src/terminal_view/README.md)** - 視圖渲染和顯示系統
- **[互動模式系統](./src/interactive/README.md)** - 即時遊戲控制系統

## 架構設計

### 核心組件概覽

完整的架構說明請參考 **[源碼架構文檔](./src/README.md)**

- **BackendManager** - 自動管理後端程序生命週期，輸出重定向到 `backend.log`
- **GameClient** - 遊戲客戶端核心，協調所有子系統
- **MqttHandler** - MQTT 網絡通信層
- **GameState** - 本地遊戲狀態管理
- **TerminalView** - 終端視圖渲染系統 [詳細文檔](./src/terminal_view/README.md)
- **Interactive** - 互動模式控制系統 [詳細文檔](./src/interactive/README.md)
- **Config** - 配置管理系統
- **Player** - 玩家和英雄定義

### MQTT 訊息格式

#### 監聽主題
- `td/+/send`: 遊戲狀態更新
- `ability_test/response`: 能力測試回應

#### 發送主題
- `td/{player_name}/action`: 玩家操作

#### 訊息格式
```json
{
  "t": "player_action",
  "a": "move",
  "d": {
    "target_x": 300.0,
    "target_y": 200.0
  }
}
```

## 配置

### 配置文件 (`config.toml`)

omobaf 使用 `config.toml` 作為主要配置文件：

```toml
[server]
# MQTT 服務器配置
mqtt_host = "127.0.0.1"  # MQTT Broker 地址
mqtt_port = 1883          # MQTT Broker 端口

[backend]
# 後端執行檔路徑（相對或絕對路徑）
executable_path = "../omobab/target/debug/omobab.exe"
# 或使用 release 版本
# executable_path = "../omobab/target/release/omobab.exe"

# 後端啟動參數
args = []

# 工作目錄（可選，預設為後端執行檔所在目錄）
working_directory = "../omobab"

# 環境變數設定（可選）
[backend.env]
RUST_LOG = "info"
# RUST_BACKTRACE = "1"

[frontend]
# 前端設定
player_name = "TestPlayer"
hero_type = "saika_magoichi"  # 或 "date_masamune"

# 自動啟動後端（重要功能）
auto_start_backend = true

# 後端啟動延遲（毫秒）- 等待後端初始化
backend_start_delay = 1000

# 後端關閉等待時間（毫秒）
backend_shutdown_timeout = 5000
```

### 命令行參數

命令行參數會覆蓋配置文件設定：

- `--server-ip`: MQTT 服務器 IP
- `--server-port`: MQTT 服務器端口
- `--client-id`: MQTT 客戶端 ID
- `--player-name`: 玩家名稱
- `--hero`: 英雄類型
- `--verbose`: 詳細日誌輸出
- `--no-auto-backend`: 禁用自動啟動後端

## 工作流程

### 啟動流程

1. **讀取配置**：從 `config.toml` 載入配置
2. **後端管理**：
   - 如果 `auto_start_backend = true`，自動啟動 omobab 後端
   - 清理舊的後端進程（Windows: taskkill，Unix: pkill）
   - 將後端輸出重定向到 `backend.log`
   - 等待後端初始化（預設 1000ms）
3. **MQTT 連接**：連接到 MQTT Broker
4. **遊戲初始化**：初始化遊戲狀態和視圖系統
5. **進入遊戲循環**：處理用戶輸入和服務器訊息

### 關閉流程

1. 保存遊戲狀態（如需要）
2. 斷開 MQTT 連接
3. 如果後端是由前端啟動的，自動關閉後端程序
4. 清理所有 omobab 進程（確保沒有遺留進程）

### 訊息流程

```
用戶輸入 → CLI/Interactive → GameClient → MqttHandler → MQTT Broker
                                ↓
                            GameState
                                ↓
                          TerminalView → 終端顯示

MQTT Broker → MqttHandler → GameState → TerminalView → 終端更新
```

## 開發和除錯

### 日誌系統

```bash
# 詳細日誌
omobaf --verbose play --hero saika_magoichi

# 或設置環境變數
RUST_LOG=debug omobaf play --hero saika_magoichi

# 查看後端日誌
tail -f backend.log
```

### 多客戶端測試

```bash
# 終端 1
omobaf play --player-name "Player1" --client-id "client_1" --hero saika_magoichi

# 終端 2  
omobaf play --player-name "Player2" --client-id "client_2" --hero date_masamune
```

### 調試技巧

1. **後端日誌**：所有後端輸出都在 `backend.log`
2. **前端日誌**：使用 `--verbose` 或 `RUST_LOG=debug`
3. **MQTT 監控**：使用 mosquitto_sub 監控訊息
   ```bash
   mosquitto_sub -h 127.0.0.1 -t "td/+/send" -v
   ```
4. **狀態檢查**：使用 `status` 命令查看遊戲狀態

## 故障排除

### 常見問題

1. **連接失敗**
   - 檢查 MQTT Broker 是否運行
   - 驗證 IP 和端口設定
   - 檢查網路連接

2. **技能施放失敗**
   - 確認技能 ID 正確
   - 檢查英雄類型匹配
   - 查看服務器日誌

3. **狀態同步問題**
   - 檢查 MQTT 訊息格式
   - 確認主題訂閱正確
   - 查看同步錯誤計數

### 日誌檢查

```bash
# 查看詳細日誌
RUST_LOG=debug omobaf status --verbose
```

## 特殊功能

### 終端視圖系統

omobaf 實現了完整的終端視圖系統，可在命令行中顯示遊戲狀態：

```
╔══════════════════════════════════════════════════════════════╗
║                        Game View                              ║
╠══════════════════════════════════════════════════════════════╣
║                                                               ║
║              @  (Player)                                      ║
║              S  (雜賀眾召喚物)                                ║
║              *  (其他玩家)                                    ║
║                                                               ║
╠══════════════════════════════════════════════════════════════╣
║ Player: TestPlayer | HP: 100/100 | MP: 50/50 | Pos: (400,300)║
╚══════════════════════════════════════════════════════════════╝
```

視圖功能：
- **視口跟隨**：自動跟隨玩家移動
- **坐標系統**：地圖大小 800x600，視口自動調整
- **實體顯示**：不同符號代表不同實體類型
- **狀態欄**：顯示玩家血量、魔力、位置等信息

### 互動模式

進入互動模式後，可使用以下快捷鍵：
- `w/a/s/d`: 移動角色
- `q/e/r/f`: 施放技能
- `space`: 基礎攻擊
- `v`: 切換視圖模式
- `ESC`: 退出

### 自動化測試

omobaf 支援多種自動化測試模式：

1. **自動遊戲模式** (`auto` 命令)
   - 自動移動、攻擊、施放技能
   - 可設定運行時長

2. **演示模式** (`demo` 命令)
   - 執行預設的操作序列
   - 展示所有功能

3. **批量測試**
   - 可編寫腳本批量執行測試
   - 支援多客戶端並發測試

## 技術規格

- **語言**: Rust 2021 Edition
- **異步運行時**: Tokio
- **MQTT 客戶端**: rumqttc
- **CLI 框架**: clap 4.0
- **序列化**: serde + serde_json
- **數學庫**: vek
- **日誌**: log + env_logger
- **終端控制**: crossterm
- **配置管理**: toml

## 授權

與 `omobab` 專案相同的開源授權。

## 貢獻

歡迎提交 Issue 和 Pull Request 來改進這個專案。

---

**注意**: 這是一個用於測試和開發的工具，不適用於生產環境。