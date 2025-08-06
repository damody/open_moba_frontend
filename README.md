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

## 架構設計

### 核心組件

- **GameClient**: 遊戲客戶端核心，處理 MQTT 連接和遊戲邏輯
- **MqttHandler**: MQTT 訊息處理器，解析和路由遊戲訊息
- **GameState**: 本地遊戲狀態管理，維護狀態同步
- **PlayerSimulator**: 玩家操作模擬器，生成和執行遊戲操作
- **CLI**: 命令行介面，提供用戶交互

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

### 命令行參數

- `--server-ip`: MQTT 服務器 IP（預設: 45.32.32.40）
- `--server-port`: MQTT 服務器端口（預設: 1883）
- `--client-id`: MQTT 客戶端 ID（預設: omobaf_player）
- `--player-name`: 玩家名稱（預設: TestPlayer）
- `--hero`: 英雄類型（預設: saika_magoichi）
- `--verbose`: 詳細日誌輸出

### 未來配置文件支援

計劃支援 `omobaf.toml` 配置文件：

```toml
[connection]
server_ip = "45.32.32.40"
server_port = 1883
client_id = "omobaf_player_001"

[player]
default_hero = "saika_magoichi"
auto_play = false
operation_interval_ms = 1000
```

## 開發和除錯

### 日誌級別

```bash
# 詳細日誌
omobaf --verbose play --hero saika_magoichi

# 或設置環境變數
RUST_LOG=debug omobaf play --hero saika_magoichi
```

### 多客戶端測試

```bash
# 終端 1
omobaf play --player-name "Player1" --client-id "client_1" --hero saika_magoichi

# 終端 2  
omobaf play --player-name "Player2" --client-id "client_2" --hero date_masamune
```

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

## 技術規格

- **語言**: Rust 2021 Edition
- **異步運行時**: Tokio
- **MQTT 客戶端**: rumqttc
- **CLI 框架**: clap 4.0
- **序列化**: serde + serde_json
- **數學庫**: vek
- **日誌**: log + env_logger

## 授權

與 `omobab` 專案相同的開源授權。

## 貢獻

歡迎提交 Issue 和 Pull Request 來改進這個專案。

---

**注意**: 這是一個用於測試和開發的工具，不適用於生產環境。