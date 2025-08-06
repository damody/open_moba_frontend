/// 輸入處理模塊
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

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
}

impl InputHandler {
    /// 創建新的輸入處理器
    pub fn new() -> Self {
        let exit_flag = Arc::new(AtomicBool::new(false));
        let exit_flag_clone = exit_flag.clone();
        
        // 啟動專用的退出鍵檢測線程
        let input_thread = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(50));
                
                // 方法1: Windows API 直接檢測按鍵
                #[cfg(windows)]
                {
                    unsafe {
                        // 檢測 ESC 鍵
                        if GetAsyncKeyState(VK_ESCAPE) & (0x8000u16 as i16) != 0 {
                            std::fs::write("debug_key.txt", "ESC key detected via WinAPI!").ok();
                            exit_flag_clone.store(true, Ordering::Relaxed);
                            return;
                        }
                        
                        // 檢測 'Q' 鍵 (VK code 81)
                        if GetAsyncKeyState(81) & (0x8000u16 as i16) != 0 {
                            std::fs::write("debug_key.txt", "Q key detected via WinAPI!").ok();
                            exit_flag_clone.store(true, Ordering::Relaxed);
                            return;
                        }
                        
                        // 檢測 Ctrl+C (VK_CONTROL + 'C')
                        if (GetAsyncKeyState(0x11) & (0x8000u16 as i16) != 0) && (GetAsyncKeyState(67) & (0x8000u16 as i16) != 0) {
                            std::fs::write("debug_key.txt", "Ctrl+C detected via WinAPI!").ok();
                            exit_flag_clone.store(true, Ordering::Relaxed);
                            return;
                        }
                    }
                }
                
                // 方法2: 嘗試 crossterm (作為備選)
                if let Ok(true) = event::poll(Duration::from_millis(0)) {
                    if let Ok(Event::Key(key_event)) = event::read() {
                        match key_event.code {
                            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                                std::fs::write("debug_key.txt", "Ctrl+C detected via crossterm!").ok();
                                exit_flag_clone.store(true, Ordering::Relaxed);
                                return;
                            },
                            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                                std::fs::write("debug_key.txt", format!("Exit key detected via crossterm: {:?}", key_event)).ok();
                                exit_flag_clone.store(true, Ordering::Relaxed);
                                return;
                            },
                            _ => {
                                std::fs::write("debug_key.txt", format!("Other key via crossterm: {:?}", key_event)).ok();
                            }
                        }
                    }
                }
            }
        });
        
        Self {
            selected_ability: None,
            exit_requested: exit_flag,
            input_thread: Some(input_thread),
        }
    }
    
    /// 等待用戶按鍵
    pub fn wait_for_key(&self) -> io::Result<KeyEvent> {
        loop {
            if let Event::Key(key_event) = event::read()? {
                return Ok(key_event);
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
    fn handle_key_event(&mut self, key_event: KeyEvent, game_state: &GameState) -> io::Result<UserInput> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
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
    fn handle_mouse_event(
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