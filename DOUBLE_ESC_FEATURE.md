# 雙擊 ESC 退出功能

## 概述

這個功能讓用戶可以通過連按 ESC 鍵兩次來快速退出程式，並且會自動關閉由 `omf` 啟動的 `omb` 後端程序。

## 功能特點

### 1. 雙擊 ESC 退出
- 第一次按 ESC：顯示提示信息，不會立即退出
- 500ms 內再按一次 ESC：立即退出程式
- 超過 500ms：重置計數器，重新開始

### 2. 智能提示系統
- 顯示當前 ESC 按鍵狀態
- 倒數計時顯示剩餘時間
- 視覺化提示用戶操作

### 3. 自動後端清理
- 退出時自動停止由 `omf` 啟動的 `omb` 後端
- 確保沒有殭屍進程
- 優雅關閉所有連接

## 技術實現

### 1. 按鍵計數器
```rust
pub struct InputHandler {
    /// ESC 按鍵計數器
    esc_count: u32,
    /// 上次 ESC 按鍵時間
    last_esc_time: std::time::Instant,
    /// 雙擊時間窗口（毫秒）
    double_click_window: u64,
}
```

### 2. 時間窗口檢測
```rust
fn handle_esc_key(&mut self) -> io::Result<UserInput> {
    let now = std::time::Instant::now();
    let time_since_last_esc = now.duration_since(self.last_esc_time).as_millis() as u64;
    
    if time_since_last_esc <= self.double_click_window {
        // 在時間窗口內，增加計數
        self.esc_count += 1;
        if self.esc_count >= 2 {
            // 雙擊 ESC，退出程式
            return Ok(UserInput::Quit);
        }
    } else {
        // 超過時間窗口，重置計數
        self.esc_count = 1;
    }
    // ...
}
```

### 3. 狀態提示
```rust
pub fn get_esc_status(&self) -> String {
    let now = std::time::Instant::now();
    let time_since_last_esc = now.duration_since(self.last_esc_time).as_millis() as u64;
    
    if self.esc_count > 0 && time_since_last_esc <= self.double_click_window {
        format!("再按一次 ESC 退出 ({}ms 內)", self.double_click_window - time_since_last_esc)
    } else {
        "按 ESC 兩次快速退出".to_string()
    }
}
```

## 使用方式

### 1. 正常退出
- 按 `q` 或 `Q`：立即退出
- 按 `Ctrl+C`：立即退出
- 輸入 `exit` 或 `quit` 命令：立即退出

### 2. 雙擊 ESC 退出
- 第一次按 ESC：顯示 "再按一次 ESC 退出 (XXXms 內)"
- 500ms 內再按一次 ESC：立即退出
- 超過 500ms：重新開始計數

### 3. 技能模式下的 ESC
- 如果正在選擇技能，第一次按 ESC 會取消技能選擇
- 第二次按 ESC 才會觸發退出邏輯

## 配置選項

### 時間窗口設定
```rust
double_click_window: 500, // 500ms 內按兩次 ESC 才算雙擊
```

可以根據需要調整這個值：
- 更短（如 300ms）：需要更快反應
- 更長（如 1000ms）：給用戶更多時間

## 視覺提示

### 1. 狀態顯示
- 在終端底部顯示 ESC 狀態
- 使用黃色文字突出顯示
- 包含倒數計時信息

### 2. 提示信息
- "按 ESC 兩次快速退出"：初始狀態
- "再按一次 ESC 退出 (XXXms 內)"：等待第二次按鍵
- 動態更新剩餘時間

## 後端清理

### 1. 自動清理
```rust
async fn handle_exit(&mut self) -> Result<()> {
    // 停止後端程序（如果由我們管理的話）
    if let Some(ref backend_manager) = self.command_handler.backend_manager {
        if backend_manager.is_running().await {
            println!("{} 停止後端程序...", "→".yellow());
            backend_manager.stop().await?;
        }
    }
    // ...
}
```

### 2. 清理範圍
- 停止 MQTT 連接
- 關閉後端進程
- 清理臨時文件
- 重置所有狀態

## 測試

### 運行測試腳本
```bash
./test_double_esc.sh
```

### 手動測試步驟
1. 啟動 `omf`
2. 進入遊戲模式
3. 按一次 ESC，觀察提示
4. 在 500ms 內再按一次 ESC
5. 確認程式退出且後端關閉

### 測試要點
- ✅ 第一次按 ESC 顯示提示
- ✅ 500ms 內再按一次 ESC 退出
- ✅ 超過 500ms 重置計數
- ✅ 退出時自動關閉後端
- ✅ 提示信息正確顯示

## 故障排除

### 1. ESC 鍵無響應
- 檢查終端是否支援 ESC 鍵檢測
- 確認 crossterm 庫正常工作
- 檢查輸入處理線程是否運行

### 2. 後端未關閉
- 檢查 `backend_manager` 是否正確初始化
- 確認後端進程 ID 是否正確
- 查看日誌中的錯誤信息

### 3. 提示不顯示
- 檢查終端尺寸是否足夠
- 確認渲染函數是否正常調用
- 檢查顏色支援是否正常

## 未來擴展

- 支援自定義時間窗口
- 支援其他按鍵組合（如 Ctrl+ESC）
- 支援聲音提示
- 支援更豐富的視覺效果