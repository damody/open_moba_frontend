# View 模式退出功能修復

## 問題描述

在 view 模式中，用戶無法通過雙擊 ESC 鍵退出，即使在其他模式中這個功能正常工作。

## 問題原因

1. **輸入處理差異**：view 模式使用 `render_live` 函數，它調用 `handle_input` 進行非阻塞輸入處理
2. **非阻塞模式限制**：`handle_input` 使用 `event::poll(Duration::from_millis(0))` 進行非阻塞檢查
3. **雙擊邏輯失效**：第一次按 ESC 返回 `UserInput::Continue`，函數立即返回，沒有等待第二次按鍵

## 修復方案

### 1. 創建專用的 view 模式輸入處理

```rust
/// 處理 view 模式的輸入（支援雙擊 ESC）
fn handle_view_input(&mut self, game_state: &GameState) -> io::Result<UserInput> {
    // 首先檢查退出標誌
    if self.input_handler.is_exit_requested() {
        return Ok(UserInput::Quit);
    }
    
    // 檢查是否有輸入事件
    if crossterm::event::poll(std::time::Duration::from_millis(0))? {
        match crossterm::event::read()? {
            crossterm::event::Event::Key(key_event) => {
                return self.input_handler.handle_key_event(key_event, game_state);
            },
            crossterm::event::Event::Mouse(mouse_event) => {
                return self.input_handler.handle_mouse_event(
                    mouse_event,
                    game_state,
                    &self.viewport,
                    self.terminal_width,
                    self.terminal_height
                );
            },
            _ => {} // 忽略其他事件
        }
    }
    
    Ok(UserInput::Continue)
}
```

### 2. 修改 render_live 函數

```rust
/// 實時模式循環
pub fn render_live(&mut self, game_state: &GameState) -> io::Result<UserInput> {
    // 渲染當前狀態
    self.render(game_state)?;
    
    // 顯示 ESC 狀態提示
    self.render_esc_status()?;
    
    // 在 view 模式下使用特殊的輸入處理
    self.handle_view_input(game_state)
}
```

### 3. 公開必要的函數

```rust
impl InputHandler {
    /// 處理鍵盤事件
    pub fn handle_key_event(&mut self, key_event: KeyEvent, game_state: &GameState) -> io::Result<UserInput>
    
    /// 處理滑鼠事件
    pub fn handle_mouse_event(&mut self, mouse_event: MouseEvent, ...) -> io::Result<UserInput>
    
    /// 檢查是否請求退出
    pub fn is_exit_requested(&self) -> bool
}
```

## 修復效果

### 修復前
- view 模式中第一次按 ESC 顯示提示
- 但無法等待第二次按鍵，因為函數立即返回
- 雙擊 ESC 退出功能失效

### 修復後
- view 模式中第一次按 ESC 顯示提示
- 500ms 內再按一次 ESC 正確退出
- 雙擊 ESC 退出功能正常工作
- 保持與其他模式一致的行為

## 測試方法

### 1. 手動測試
```bash
./test_simple_view_exit.sh
```

### 2. 自動測試
```bash
./test_view_mode_exit.sh
```

### 3. 測試步驟
1. 啟動 `omf`
2. 連接後端：`connect localhost`
3. 進入遊戲：`play`
4. 進入視圖模式：`view`
5. 測試雙擊 ESC 退出

## 技術細節

### 1. 輸入處理流程
```
render_live() 
  → handle_view_input()
    → 檢查退出標誌
    → 非阻塞檢查輸入事件
    → 調用 handle_key_event()
      → handle_esc_key()
        → 雙擊邏輯處理
```

### 2. 狀態管理
- `esc_count`：ESC 按鍵計數
- `last_esc_time`：上次按鍵時間
- `double_click_window`：雙擊時間窗口（500ms）
- `exit_requested`：退出請求標誌

### 3. 提示系統
- 在終端底部顯示 ESC 狀態
- 動態更新剩餘時間
- 視覺化提示用戶操作

## 相關文件

- `src/terminal_view/mod.rs`：主要修復文件
- `src/terminal_view/input.rs`：輸入處理邏輯
- `test_view_mode_exit.sh`：自動測試腳本
- `test_simple_view_exit.sh`：手動測試腳本

## 注意事項

1. 修復保持了向後兼容性
2. 不影響其他模式的正常功能
3. 雙擊時間窗口可配置（目前為 500ms）
4. 支援所有現有的退出方式（q、Ctrl+C、exit 命令等）