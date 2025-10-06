# 單擊 ESC 退出功能

## 概述

根據用戶反饋，將原本的雙擊 ESC 退出功能簡化為單擊 ESC 退出，提供更直接和簡單的退出方式。

## 功能特點

### 1. 單擊 ESC 退出
- 按一次 ESC 鍵立即退出程式
- 不需要等待第二次按鍵
- 更直觀和快速

### 2. 智能處理
- 如果正在選擇技能，ESC 會取消技能選擇
- 如果沒有選擇技能，ESC 會退出程式
- 保持原有的功能邏輯

### 3. 簡化提示
- 提示信息更簡潔明了
- 根據當前狀態顯示不同的提示

## 技術實現

### 1. 簡化的 ESC 處理
```rust
/// 處理 ESC 鍵（單擊退出）
fn handle_esc_key(&mut self) -> io::Result<UserInput> {
    if self.selected_ability.is_some() {
        // 如果有選擇的技能，取消技能選擇
        self.selected_ability = None;
        Ok(UserInput::Cancel)
    } else {
        // 直接退出程式
        self.exit_requested.store(true, Ordering::Relaxed);
        Ok(UserInput::Quit)
    }
}
```

### 2. 動態提示信息
```rust
/// 獲取 ESC 按鍵狀態（用於顯示提示）
pub fn get_esc_status(&self) -> String {
    if self.selected_ability.is_some() {
        "按 ESC 取消技能選擇".to_string()
    } else {
        "按 ESC 退出程式".to_string()
    }
}
```

### 3. 移除複雜邏輯
- 移除了雙擊計數器
- 移除了時間窗口檢測
- 移除了複雜的狀態管理

## 使用方式

### 1. 正常退出
- 按 `ESC`：立即退出（如果沒有選擇技能）
- 按 `q` 或 `Q`：立即退出
- 按 `Ctrl+C`：立即退出
- 輸入 `exit` 或 `quit` 命令：立即退出

### 2. 技能模式
- 如果正在選擇技能，按 `ESC` 會取消技能選擇
- 再次按 `ESC` 才會退出程式

### 3. View 模式
- 在 view 模式中按 `ESC` 立即退出視圖模式
- 回到主選單後可以再次按 `ESC` 完全退出

## 測試

### 運行測試腳本
```bash
./test_single_esc_exit.sh
```

### 測試步驟
1. 啟動 `omf`
2. 連接後端：`connect localhost`
3. 進入遊戲：`play`
4. 進入視圖模式：`view`
5. 測試單擊 ESC 退出

### 測試要點
- ✅ 按一次 ESC 就退出，不需要雙擊
- ✅ 提示信息正確顯示
- ✅ 技能選擇時 ESC 取消選擇
- ✅ 所有模式都支援單擊 ESC 退出

## 優勢

### 1. 簡化操作
- 減少用戶操作步驟
- 降低學習成本
- 提高使用效率

### 2. 更直觀
- 符合常見軟體的使用習慣
- 減少用戶困惑
- 提供即時反饋

### 3. 減少錯誤
- 避免雙擊時間窗口問題
- 減少意外操作
- 提高穩定性

## 向後兼容

- 保持所有現有的退出方式
- 不影響其他功能
- 保持相同的退出行為

## 相關文件

- `src/terminal_view/input.rs`：主要修改文件
- `test_single_esc_exit.sh`：測試腳本
- `SINGLE_ESC_EXIT.md`：本說明文件

## 注意事項

1. 這個修改影響所有使用 ESC 鍵的地方
2. 包括 view 模式、互動模式等
3. 保持了技能選擇的取消功能
4. 提示信息會根據當前狀態動態更新