/// 輸入處理模塊
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use log::debug;
use crate::terminal_logger::TerminalLogger;

#[cfg(windows)]
use winapi::um::winuser::{GetAsyncKeyState, VK_ESCAPE};
use vek::Vec2;
use crate::game_state::GameState;
use super::viewport::ViewportManager;

/// 用戶輸入事件
#[derive(Debug, Clone)]
pub enum UserInput {
    /// 退出事件
    Quit,
    /// 滑鼠左鍵點擊移動 (世界座標)
    Move(Vec2<f32>),
    /// 滑鼠右鍵點擊攻擊 (世界座標)
    Attack(Vec2<f32>),
    /// Shift+左鍵點擊移動攻擊 (世界座標)
    MoveAttack(Vec2<f32>),
    /// Ctrl+左鍵點擊強制攻擊 (世界座標，包括友軍)
    ForceAttack(Vec2<f32>),
    /// 施放技能 (技能ID, 世界座標)
    CastAbility(String, Vec2<f32>),
    /// 使用道具 (道具ID, 世界座標)
    UseItem(String, Option<Vec2<f32>>),
    /// 取消當前操作
    Cancel,
    /// 繼續循環
    Continue,
}

/// 輸入處理器
pub struct InputHandler {
    /// 當前選擇的技能（技能模式）
    pub selected_ability: Option<String>,
    /// 退出標誌
    exit_requested: Arc<AtomicBool>,
    /// 輸入線程句柄
    input_thread: Option<thread::JoinHandle<()>>,
    /// Linux: 從背景執行緒接收事件的通道（非阻塞讀取）
    #[cfg(not(windows))]
    event_rx: Option<Receiver<Event>>,
    /// Linux: 停止背景執行緒的旗標
    #[cfg(not(windows))]
    stop_flag: Option<Arc<AtomicBool>>,
}

impl InputHandler {
    /// 創建新的輸入處理器
    pub fn new() -> Self {
        let exit_flag = Arc::new(AtomicBool::new(false));

        // 僅在 Windows 上啟動後台鍵盤檢測線程（使用 WinAPI），
        // 以避免在 Linux 上和主循環同時讀取 crossterm 事件造成事件被搶讀。
        #[cfg(windows)]
        let input_thread = {
            let exit_flag_clone = exit_flag.clone();
            Some(thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_millis(50));
                    unsafe {
                        // 檢測 ESC 鍵
                        if GetAsyncKeyState(VK_ESCAPE) & (0x8000u16 as i16) != 0 {
                            exit_flag_clone.store(true, Ordering::Relaxed);
                            return;
                        }
                        // 檢測 'Q' 鍵 (VK code 81)
                        if GetAsyncKeyState(81) & (0x8000u16 as i16) != 0 {
                            exit_flag_clone.store(true, Ordering::Relaxed);
                            return;
                        }
                        // 檢測 Ctrl+C (VK_CONTROL + 'C')
                        if (GetAsyncKeyState(0x11) & (0x8000u16 as i16) != 0)
                            && (GetAsyncKeyState(67) & (0x8000u16 as i16) != 0)
                        {
                            exit_flag_clone.store(true, Ordering::Relaxed);
                            return;
                        }
                    }
                }
            }))
        };

        #[cfg(not(windows))]
        {
            // Linux: 在 new() 就建立事件通道與背景執行緒
            let (tx, rx) = mpsc::channel::<Event>();
            let stop_flag = Arc::new(AtomicBool::new(false));
            let stop_flag_clone = stop_flag.clone();

            let handle = thread::spawn(move || {
                loop {
                    // 每 50ms 檢查是否有事件，並允許響應停止旗標
                    if stop_flag_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    match event::poll(Duration::from_millis(50)) {
                        Ok(true) => {
                            match event::read() {
                                Ok(ev) => { let _ = tx.send(ev); }
                                Err(_) => thread::sleep(Duration::from_millis(5)),
                            }
                        }
                        Ok(false) => { /* no event; loop to check stop flag */ }
                        Err(_) => thread::sleep(Duration::from_millis(5)),
                    }
                }
            });

            return Self {
                selected_ability: None,
                exit_requested: exit_flag,
                input_thread: Some(handle),
                event_rx: Some(rx),
                stop_flag: Some(stop_flag),
            };
        }

        #[cfg(windows)]
        return Self { selected_ability: None, exit_requested: exit_flag, input_thread };
    }

    /// 在 Linux 上啟動背景事件讀取執行緒（阻塞 read，主循環非阻塞 try_recv）
    #[cfg(not(windows))]
    pub fn start_event_thread(&mut self) {
        // 已啟動則略過
        if self.event_rx.is_some() {
            return;
        }

        let (tx, rx) = mpsc::channel::<Event>();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        // 背景執行緒：阻塞讀取事件並送入通道
        let handle = thread::spawn(move || {
            loop {
                // 若要求停止，嘗試優雅退出（需要有事件或下一輪檢查）
                if stop_flag_clone.load(Ordering::Relaxed) {
                    break;
                }
                match event::read() {
                    Ok(ev) => {
                        let _ = tx.send(ev);
                    }
                    Err(_) => {
                        // 避免忙迴圈
                        thread::sleep(Duration::from_millis(5));
                    }
                }
            }
        });

        self.event_rx = Some(rx);
        self.stop_flag = Some(stop_flag);
        self.input_thread = Some(handle);
    }

    /// 嘗試非阻塞取得一個事件
    #[cfg(not(windows))]
    pub fn try_recv_event(&self) -> Option<Event> {
        if let Some(rx) = &self.event_rx {
            match rx.try_recv() {
                Ok(ev) => Some(ev),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => None,
            }
        } else {
            None
        }
    }

    /// 嘗試停止事件讀取執行緒（注意：若 read 阻塞，可能延後生效）
    #[cfg(not(windows))]
    pub fn stop_event_thread(&mut self) {
        if let Some(flag) = &self.stop_flag {
            flag.store(true, Ordering::Relaxed);
        }
        if let Some(handle) = self.input_thread.take() {
            // 嘗試加入，避免長時間阻塞
            let _ = handle.join();
        }
        self.event_rx = None;
        self.stop_flag = None;
    }
    
    /// 等待用戶按鍵
    pub fn wait_for_key(&self) -> io::Result<KeyEvent> {
        #[cfg(not(windows))]
        {
            // 從背景執行緒的通道阻塞接收事件，避免與背景讀取競爭
            if let Some(rx) = &self.event_rx {
                loop {
                    match rx.recv() {
                        Ok(Event::Key(key_event)) => return Ok(key_event),
                        Ok(_) => continue, // 忽略非鍵盤事件
                        Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "event channel closed")),
                    }
                }
            }
            // 如果沒有通道（理論上不會發生），退回同步 read
            loop {
                if let Event::Key(key_event) = event::read()? {
                    return Ok(key_event);
                }
            }
        }

        #[cfg(windows)]
        {
            loop {
                if let Event::Key(key_event) = event::read()? {
                    return Ok(key_event);
                }
            }
        }
    }
    
    /// 處理用戶輸入（非阻塞）
    pub fn handle_input(
        &mut self,
        game_state: &GameState,
        viewport: &ViewportManager,
        terminal_width: u16,
        terminal_height: u16,
    ) -> io::Result<UserInput> {
        // 首先檢查退出標誌
        if self.exit_requested.load(Ordering::Relaxed) {
            return Ok(UserInput::Quit);
        }
        
        // 然後檢查其他輸入事件
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key_event) => {
                    // 在底部日誌輸出捕獲的按鍵（Linux 調試）
                    crate::terminal_logger::TerminalLogger::global()
                        .log("DEBUG", format!("key: {:?} mods: {:?}", key_event.code, key_event.modifiers));
                    return self.handle_key_event(key_event, game_state);
                },
                Event::Mouse(mouse_event) => {
                    return self.handle_mouse_event(
                        mouse_event,
                        game_state,
                        viewport,
                        terminal_width,
                        terminal_height
                    );
                },
                _ => {} // 忽略其他事件
            }
        }
        
        Ok(UserInput::Continue)
    }
    
    /// 處理鍵盤事件
    pub fn handle_key_event(&mut self, key_event: KeyEvent, game_state: &GameState) -> io::Result<UserInput> {
        match key_event.code {
            KeyCode::Esc => {
                self.handle_esc_key()
            },
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if self.selected_ability.is_some() {
                    // 取消技能選擇
                    self.selected_ability = None;
                    Ok(UserInput::Cancel)
                } else {
                    // 設置退出標誌
                    self.exit_requested.store(true, Ordering::Relaxed);
                    Ok(UserInput::Quit)
                }
            },
            // 技能快捷鍵 - W/E/R/T 對應當前英雄的技能
            KeyCode::Char('w') | KeyCode::Char('W') => {
                if let Some(ability) = self.get_hero_ability(game_state, 0) {
                    self.selected_ability = Some(ability);
                }
                Ok(UserInput::Continue)
            },
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(ability) = self.get_hero_ability(game_state, 1) {
                    self.selected_ability = Some(ability);
                }
                Ok(UserInput::Continue)
            },
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if let Some(ability) = self.get_hero_ability(game_state, 2) {
                    self.selected_ability = Some(ability);
                }
                Ok(UserInput::Continue)
            },
            KeyCode::Char('t') | KeyCode::Char('T') => {
                if let Some(ability) = self.get_hero_ability(game_state, 3) {
                    self.selected_ability = Some(ability);
                }
                Ok(UserInput::Continue)
            },
            // 道具快捷鍵 - 數字鍵 1-9
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                let slot = c.to_digit(10).unwrap() as u8;
                if let Some(item) = self.get_item_by_slot(game_state, slot) {
                    Ok(UserInput::UseItem(item.item_id.clone(), None))
                } else {
                    Ok(UserInput::Continue)
                }
            },
            _ => Ok(UserInput::Continue)
        }
    }
    
    /// 處理滑鼠事件
    pub fn handle_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        game_state: &GameState,
        viewport: &ViewportManager,
        terminal_width: u16,
        terminal_height: u16,
    ) -> io::Result<UserInput> {
        // 計算世界座標
        let world_pos = viewport.screen_to_world(
            mouse_event.column,
            mouse_event.row,
            game_state.local_player.position,
            terminal_width as usize,
            terminal_height as usize,
        );
        
        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // 如果有選擇的技能，施放技能
                if let Some(ability_id) = &self.selected_ability {
                    let result = UserInput::CastAbility(ability_id.clone(), world_pos);
                    self.selected_ability = None; // 清除技能選擇
                    return Ok(result);
                }
                
                // 檢查修飾鍵
                if mouse_event.modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift+左鍵 = 移動攻擊
                    Ok(UserInput::MoveAttack(world_pos))
                } else if mouse_event.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+左鍵 = 強制攻擊
                    Ok(UserInput::ForceAttack(world_pos))
                } else {
                    // 普通左鍵 = 移動
                    Ok(UserInput::Move(world_pos))
                }
            },
            MouseEventKind::Down(MouseButton::Right) => {
                // 如果有選擇的技能，取消選擇
                if self.selected_ability.is_some() {
                    self.selected_ability = None;
                    Ok(UserInput::Cancel)
                } else {
                    // 右鍵點擊 = 攻擊
                    Ok(UserInput::Attack(world_pos))
                }
            },
            _ => Ok(UserInput::Continue)
        }
    }
    
    /// 根據道具欄位置獲取道具
    fn get_item_by_slot<'a>(&self, game_state: &'a GameState, slot: u8) -> Option<&'a crate::game_state::ItemState> {
        game_state.local_player.items.iter()
            .find(|item| item.slot == slot && item.is_available && item.charges > 0)
    }
    
    /// 處理 ESC 鍵（單擊退出）
    fn handle_esc_key(&mut self) -> io::Result<UserInput> {
        TerminalLogger::global().log("DEBUG", "🔍 ESC 鍵被按下".to_string());
        if self.selected_ability.is_some() {
            // 如果有選擇的技能，取消技能選擇
            TerminalLogger::global().log("DEBUG", "🔍 取消技能選擇".to_string());
            self.selected_ability = None;
            Ok(UserInput::Cancel)
        } else {
            // 直接退出程式
            TerminalLogger::global().log("DEBUG", "🔍 設置退出標誌".to_string());
            self.exit_requested.store(true, Ordering::Relaxed);
            Ok(UserInput::Quit)
        }
    }
    
    /// 獲取 ESC 按鍵狀態（用於顯示提示）
    pub fn get_esc_status(&self) -> String {
        if self.selected_ability.is_some() {
            "按 ESC 取消技能選擇".to_string()
        } else {
            "按 ESC 退出程式".to_string()
        }
    }
    
    /// 檢查是否請求退出
    pub fn is_exit_requested(&self) -> bool {
        self.exit_requested.load(Ordering::Relaxed)
    }
    
    /// 根據英雄類型和索引獲取技能ID
    fn get_hero_ability(&self, game_state: &GameState, index: usize) -> Option<String> {
        let abilities = match game_state.local_player.hero_type.as_str() {
            "saika_magoichi" => vec![
                "sniper_mode",
                "saika_reinforcements", 
                "rain_iron_cannon",
                "three_stage_technique"
            ],
            "date_masamune" => vec![
                "flame_blade",
                "fire_dash",
                "flame_assault", 
                "matchlock_gun"
            ],
            _ => return None,
        };
        
        if index < abilities.len() {
            Some(abilities[index].to_string())
        } else {
            None
        }
    }
}