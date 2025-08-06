/// 輸入處理模塊
use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
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
}

impl InputHandler {
    /// 創建新的輸入處理器
    pub fn new() -> Self {
        Self {
            selected_ability: None,
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
        // 檢查用戶輸入（非阻塞）
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) => {
                    self.handle_key_event(key_event, game_state)
                },
                Event::Mouse(mouse_event) => {
                    self.handle_mouse_event(
                        mouse_event,
                        game_state,
                        viewport,
                        terminal_width,
                        terminal_height
                    )
                },
                _ => Ok(UserInput::Continue)
            }
        } else {
            Ok(UserInput::Continue)
        }
    }
    
    /// 處理鍵盤事件
    fn handle_key_event(&mut self, key_event: KeyEvent, game_state: &GameState) -> io::Result<UserInput> {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.selected_ability.is_some() {
                    // 取消技能選擇
                    self.selected_ability = None;
                    Ok(UserInput::Cancel)
                } else {
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