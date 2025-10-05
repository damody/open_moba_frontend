# Interactive 互動模式系統

互動模式提供即時的遊戲操作體驗，讓玩家可以通過鍵盤直接控制角色。

## 🎮 系統概述

Interactive 模組實現了一個完整的即時遊戲控制系統，支援鍵盤輸入、命令解析和遊戲操作執行。

## 📁 模組結構

### `mod.rs` - 模組入口
- 定義模組公共介面
- 匯出子模組功能
- 提供 `InteractiveMode` 主結構

### `session.rs` - 會話管理
- **功能**：管理互動遊戲會話的生命週期
- **核心職責**：
  - 初始化互動環境
  - 維護會話狀態
  - 協調輸入處理和命令執行
  - 管理遊戲循環
- **會話流程**：
  ```
  初始化 → 連接服務器 → 遊戲循環 → 清理退出
           ↓
        處理輸入 → 執行命令 → 更新狀態 → 渲染畫面
  ```

### `commands.rs` - 命令處理
- **功能**：解析和執行玩家命令
- **支援命令類型**：
  
  **移動命令**：
  - `move <x> <y>` - 移動到指定坐標
  - `move up/down/left/right` - 方向移動
  - 快捷鍵：W/A/S/D
  
  **戰鬥命令**：
  - `attack <x> <y>` - 攻擊指定位置
  - `cast <ability_name>` - 施放技能
  - 快捷鍵：Q/E/R/F（技能），Space（普攻）
  
  **資訊命令**：
  - `status` - 顯示角色狀態
  - `abilities` - 顯示可用技能
  - `help` - 顯示幫助信息
  
  **視圖命令**：
  - `view` - 切換視圖模式
  - `zoom in/out` - 縮放視圖
  - `center` - 視圖回到角色

## 🎯 功能特性

### 即時控制
- **非阻塞輸入**：不中斷遊戲運行
- **按鍵映射**：可自定義快捷鍵
- **組合鍵支援**：Ctrl/Alt/Shift 組合

### 命令系統
- **命令歷史**：上下鍵瀏覽歷史命令
- **自動完成**：Tab 鍵自動完成命令
- **命令別名**：支援命令簡寫

### 狀態反饋
- **即時更新**：操作結果立即顯示
- **錯誤提示**：友好的錯誤訊息
- **冷卻提示**：技能冷卻時間顯示

## 🎮 操作指南

### 快捷鍵映射

| 按鍵 | 功能 | 說明 |
|------|------|------|
| W/A/S/D | 移動 | 上/左/下/右移動 |
| Q | 技能1 | 施放第一個技能 |
| E | 技能2 | 施放第二個技能 |
| R | 技能3 | 施放第三個技能 |
| F | 技能4 | 施放第四個技能 |
| Space | 普攻 | 基礎攻擊 |
| V | 視圖 | 切換視圖模式 |
| Tab | 記分板 | 顯示遊戲統計 |
| Enter | 聊天 | 開啟聊天輸入 |
| ESC | 退出 | 退出互動模式 |

### 命令語法

```bash
# 移動到指定位置
move 400 300

# 方向移動
move up 50

# 施放技能（雜賀孫一）
cast sniper_mode
cast saika_reinforcements 450 350

# 施放技能（伊達政宗）
cast flame_blade
cast fire_dash 500 400

# 攻擊
attack 450 350
attack nearest  # 攻擊最近敵人

# 查看狀態
status
abilities
cooldowns
```

## 🔧 技術實現

### 輸入處理流程

```rust
loop {
    // 1. 檢查鍵盤輸入
    if let Some(key) = check_input() {
        // 2. 轉換為命令
        let command = parse_command(key);
        
        // 3. 驗證命令
        if validate_command(&command) {
            // 4. 執行命令
            execute_command(command)?;
            
            // 5. 更新狀態
            update_game_state();
        }
    }
    
    // 6. 處理服務器消息
    process_server_messages();
    
    // 7. 渲染畫面
    render_frame();
}
```

### 命令執行器

```rust
pub enum Command {
    Move { x: f32, y: f32 },
    Attack { target_x: f32, target_y: f32 },
    Cast { ability: String, target: Option<(f32, f32)> },
    Status,
    Help,
    Quit,
}

impl CommandExecutor {
    pub fn execute(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::Move { x, y } => self.handle_move(x, y),
            Command::Attack { .. } => self.handle_attack(..),
            Command::Cast { .. } => self.handle_cast(..),
            // ...
        }
    }
}
```

## 📊 狀態管理

### 會話狀態
- 當前位置
- 血量/魔力
- 技能冷卻
- 目標鎖定
- 視圖模式

### 輸入緩衝
- 命令隊列
- 按鍵緩衝
- 組合鍵檢測

## 🚀 使用範例

```rust
// 創建互動會話
let mut session = InteractiveSession::new(game_client);

// 啟動互動模式
session.start().await?;

// 會話會自動處理：
// - 鍵盤輸入
// - 命令執行
// - 狀態更新
// - 畫面渲染

// 直到玩家退出
session.wait_for_exit().await;
```

## 🛠️ 配置選項

```toml
[interactive]
# 輸入設定
input_buffer_size = 10
command_history_size = 50

# 快捷鍵映射
[interactive.keybindings]
move_up = "w"
move_down = "s"
move_left = "a"
move_right = "d"
skill_1 = "q"
skill_2 = "e"
skill_3 = "r"
skill_4 = "f"
attack = "space"

# 自動功能
[interactive.auto]
auto_attack = false
smart_cast = true
quick_cast = false
```

## 📝 開發注意事項

1. **線程安全**：輸入處理在獨立線程
2. **錯誤處理**：妥善處理網絡延遲
3. **狀態同步**：確保本地和服務器狀態一致
4. **性能**：避免阻塞主遊戲循環

## 🎯 未來改進

- [ ] 支援滑鼠輸入
- [ ] 添加宏命令系統
- [ ] 實現智能施法
- [ ] 支援手把控制
- [ ] 添加錄製/重播功能
- [ ] 實現 AI 輔助操作