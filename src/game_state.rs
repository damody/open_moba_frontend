/// 遊戲狀態管理
/// 
/// 維護本地遊戲狀態副本，用於驗證後端同步
// use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use log::{info, warn, debug};
use vek::Vec2;

use crate::mqtt_handler::{PlayerState, AbilityData, SummonData};

/// 遊戲狀態管理器
#[derive(Debug, Clone)]
pub struct GameState {
    /// 本地玩家信息
    pub local_player: LocalPlayer,
    /// 其他玩家狀態
    pub other_players: HashMap<String, PlayerState>,
    /// 遊戲實體
    pub entities: HashMap<u32, Entity>,
    /// 最後更新時間
    pub last_update: SystemTime,
    /// 狀態差異計數
    pub sync_errors: u64,
    /// 虛擬螢幕範圍
    pub viewport: Viewport,
}

/// 虛擬螢幕範圍
#[derive(Debug, Clone)]
pub struct Viewport {
    /// 螢幕中心位置（通常跟隨玩家）
    pub center: Vec2<f32>,
    /// 螢幕寬度
    pub width: f32,
    /// 螢幕高度
    pub height: f32,
    /// 縮放比例
    pub zoom: f32,
}

/// 本地玩家狀態
#[derive(Debug, Clone)]
pub struct LocalPlayer {
    pub name: String,
    pub hero_type: String,
    pub position: Vec2<f32>,
    pub health: (f32, f32),  // (current, max)
    pub abilities: Vec<AbilityState>,
    pub items: Vec<ItemState>,       // 道具欄 (1-9 號位)
    pub summons: Vec<SummonState>,
    pub level: u8,
    pub experience: u32,
}

/// 技能狀態
#[derive(Debug, Clone)]
pub struct AbilityState {
    pub ability_id: String,
    pub level: u8,
    pub cooldown_remaining: f32,
    pub is_available: bool,
    pub last_used: Option<SystemTime>,
}

/// 道具狀態
#[derive(Debug, Clone)]
pub struct ItemState {
    pub item_id: String,
    pub name: String,
    pub slot: u8,           // 道具欄位置 (1-9)
    pub charges: u32,       // 使用次數
    pub cooldown_remaining: f32,
    pub is_available: bool,
    pub last_used: Option<SystemTime>,
}

/// 召喚物狀態
#[derive(Debug, Clone)]
pub struct SummonState {
    pub id: u32,
    pub unit_type: String,
    pub position: Vec2<f32>,
    pub health: (f32, f32),
    pub state: SummonAIState,
    pub spawn_time: SystemTime,
}

/// 召喚物 AI 狀態
#[derive(Debug, Clone, PartialEq)]
pub enum SummonAIState {
    Idle,
    Attacking(u32),  // 攻擊目標 ID
    Moving(Vec2<f32>),  // 移動到位置
    Following,  // 跟隨主人
    Dead,
}

/// 遊戲實體
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: u32,
    pub entity_type: EntityType,
    pub position: Vec2<f32>,
    pub health: (f32, f32),
    pub owner: Option<String>,
}

/// 實體類型
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Player(String),  // 玩家名稱
    Summon(String),  // 召喚物類型
    Projectile,      // 投射物
    Effect,          // 特效
}

impl Viewport {
    /// 創建默認視窗
    pub fn default() -> Self {
        Self {
            center: Vec2::zero(),
            width: 1920.0,
            height: 1080.0,
            zoom: 1.0,
        }
    }
    
    /// 獲取視窗邊界
    pub fn get_bounds(&self) -> (Vec2<f32>, Vec2<f32>) {
        let half_width = self.width / (2.0 * self.zoom);
        let half_height = self.height / (2.0 * self.zoom);
        
        let min = Vec2::new(
            self.center.x - half_width,
            self.center.y - half_height,
        );
        let max = Vec2::new(
            self.center.x + half_width,
            self.center.y + half_height,
        );
        
        (min, max)
    }
    
    /// 更新視窗中心（跟隨玩家）
    pub fn follow_player(&mut self, player_pos: Vec2<f32>) {
        self.center = player_pos;
    }
    
    /// 設置縮放
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.5, 3.0);
    }
    
    /// 設置視窗大小
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
}

impl GameState {
    /// 創建新的遊戲狀態
    pub fn new(player_name: String, hero_type: String) -> Self {
        let local_player = LocalPlayer {
            name: player_name.clone(),
            hero_type: hero_type.clone(),
            position: Vec2::zero(),
            health: (100.0, 100.0),
            abilities: Self::init_hero_abilities(&hero_type),
            items: Self::init_default_items(),
            summons: Vec::new(),
            level: 1,
            experience: 0,
        };
        
        info!("初始化遊戲狀態 - 玩家: {}, 英雄: {}", player_name, hero_type);
        
        Self {
            local_player,
            other_players: HashMap::new(),
            entities: HashMap::new(),
            last_update: SystemTime::now(),
            sync_errors: 0,
            viewport: Viewport::default(),
        }
    }
    
    /// 初始化默認道具
    fn init_default_items() -> Vec<ItemState> {
        vec![
            ItemState {
                item_id: "health_potion".to_string(),
                name: "生命藥水".to_string(),
                slot: 1,
                charges: 5,
                cooldown_remaining: 0.0,
                is_available: true,
                last_used: None,
            },
            ItemState {
                item_id: "mana_potion".to_string(),
                name: "魔力藥水".to_string(),
                slot: 2,
                charges: 3,
                cooldown_remaining: 0.0,
                is_available: true,
                last_used: None,
            },
            ItemState {
                item_id: "teleport_scroll".to_string(),
                name: "傳送卷軸".to_string(),
                slot: 3,
                charges: 2,
                cooldown_remaining: 0.0,
                is_available: true,
                last_used: None,
            },
            ItemState {
                item_id: "smoke_bomb".to_string(),
                name: "煙霧彈".to_string(),
                slot: 4,
                charges: 4,
                cooldown_remaining: 0.0,
                is_available: true,
                last_used: None,
            },
        ]
    }
    
    /// 初始化英雄技能
    fn init_hero_abilities(hero_type: &str) -> Vec<AbilityState> {
        let ability_ids = match hero_type {
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
            _ => vec![]
        };
        
        ability_ids.into_iter().map(|id| AbilityState {
            ability_id: id.to_string(),
            level: 1,
            cooldown_remaining: 0.0,
            is_available: true,
            last_used: None,
        }).collect()
    }
    
    /// 更新玩家位置
    pub fn update_player_position(&mut self, player_name: &str, x: f32, y: f32) {
        if player_name == self.local_player.name {
            self.local_player.position = Vec2::new(x, y);
            debug!("更新本地玩家位置: ({}, {})", x, y);
        } else {
            if let Some(player) = self.other_players.get_mut(player_name) {
                player.position = (x, y);
            }
            debug!("更新其他玩家 {} 位置: ({}, {})", player_name, x, y);
        }
        
        self.last_update = SystemTime::now();
    }
    
    /// 更新玩家技能狀態
    pub fn update_player_ability(&mut self, player_name: &str, ability_data: &AbilityData) {
        if player_name == self.local_player.name {
            // 更新本地玩家技能
            if let Some(ability) = self.local_player.abilities.iter_mut()
                .find(|a| a.ability_id == ability_data.ability_id) {
                ability.level = ability_data.level;
                ability.cooldown_remaining = ability_data.cooldown_remaining;
                ability.is_available = ability_data.cooldown_remaining <= 0.0;
                ability.last_used = Some(SystemTime::now());
                
                debug!("更新本地技能狀態: {} - 冷卻: {:.1}s", 
                       ability_data.ability_id, ability_data.cooldown_remaining);
            }
        } else {
            // 更新其他玩家技能（如果需要）
            debug!("其他玩家 {} 使用技能: {}", player_name, ability_data.ability_id);
        }
        
        self.last_update = SystemTime::now();
    }
    
    /// 更新玩家生命值
    pub fn update_player_health(&mut self, player_name: &str, current: f32, max: f32) {
        if player_name == self.local_player.name {
            self.local_player.health = (current, max);
            debug!("更新本地玩家生命值: {}/{}", current, max);
        } else {
            if let Some(player) = self.other_players.get_mut(player_name) {
                player.health = (current, max);
            }
            debug!("更新其他玩家 {} 生命值: {}/{}", player_name, current, max);
        }
        
        self.last_update = SystemTime::now();
    }
    
    /// 更新召喚物狀態
    pub fn update_summon_state(&mut self, owner: &str, summon_data: &SummonData) {
        if owner == self.local_player.name {
            // 查找或創建召喚物
            let position = Vec2::new(summon_data.position.0, summon_data.position.1);
            
            // 簡單匹配：根據類型和位置找到對應的召喚物
            if let Some(summon) = self.local_player.summons.iter_mut()
                .find(|s| s.unit_type == summon_data.unit_type) {
                summon.position = position;
                summon.health.0 = summon_data.health;
                summon.state = match summon_data.state.as_str() {
                    "idle" => SummonAIState::Idle,
                    "attacking" => SummonAIState::Attacking(0), // 簡化處理
                    "moving" => SummonAIState::Moving(position),
                    "following" => SummonAIState::Following,
                    "dead" => SummonAIState::Dead,
                    _ => SummonAIState::Idle,
                };
            } else {
                // 創建新召喚物
                let new_summon = SummonState {
                    id: self.local_player.summons.len() as u32 + 1,
                    unit_type: summon_data.unit_type.clone(),
                    position,
                    health: (summon_data.health, 100.0), // 假設最大生命值
                    state: SummonAIState::Idle,
                    spawn_time: SystemTime::now(),
                };
                
                self.local_player.summons.push(new_summon);
                debug!("創建新召喚物: {} 在位置 ({}, {})", 
                       summon_data.unit_type, summon_data.position.0, summon_data.position.1);
            }
        }
        
        self.last_update = SystemTime::now();
    }
    
    /// 同步完整玩家狀態
    pub fn sync_player_state(&mut self, player_state: &PlayerState) {
        if player_state.name == self.local_player.name {
            // 驗證本地狀態與服務器狀態的一致性
            let server_pos = Vec2::new(player_state.position.0, player_state.position.1);
            let pos_diff = (self.local_player.position - server_pos).magnitude();
            
            if pos_diff > 5.0 {  // 允許 5 像素的誤差
                warn!("位置同步差異過大: 本地 {:?}, 服務器 {:?}, 差異: {:.2}", 
                      self.local_player.position, server_pos, pos_diff);
                self.sync_errors += 1;
            }
            
            // 同步服務器狀態
            self.local_player.position = server_pos;
            self.local_player.health = player_state.health;
            
            // 同步技能狀態
            for server_ability in &player_state.abilities {
                if let Some(local_ability) = self.local_player.abilities.iter_mut()
                    .find(|a| a.ability_id == server_ability.ability_id) {
                    local_ability.level = server_ability.level;
                    local_ability.cooldown_remaining = server_ability.cooldown_remaining;
                    local_ability.is_available = server_ability.cooldown_remaining <= 0.0;
                }
            }
            
            debug!("同步本地玩家狀態完成");
        } else {
            // 更新其他玩家狀態
            self.other_players.insert(player_state.name.clone(), player_state.clone());
            debug!("更新其他玩家狀態: {}", player_state.name);
        }
        
        self.last_update = SystemTime::now();
    }
    
    /// 應用本地操作
    pub fn apply_local_action(&mut self, action: &str, result: &serde_json::Value) {
        match action {
            "move" => {
                if let (Some(x), Some(y)) = (result.get("x"), result.get("y")) {
                    if let (Some(x), Some(y)) = (x.as_f64(), y.as_f64()) {
                        self.local_player.position = Vec2::new(x as f32, y as f32);
                        debug!("應用本地移動操作: ({}, {})", x, y);
                    }
                }
            },
            "cast_ability" => {
                if let Some(ability_id) = result.get("ability_id").and_then(|v| v.as_str()) {
                    if let Some(ability) = self.local_player.abilities.iter_mut()
                        .find(|a| a.ability_id == ability_id) {
                        ability.is_available = false;
                        ability.last_used = Some(SystemTime::now());
                        // 設置測試冷卻時間（實際應由服務器提供）
                        ability.cooldown_remaining = match ability_id {
                            "sniper_mode" => 8.0,
                            "saika_reinforcements" => 12.0,
                            "rain_iron_cannon" => 15.0,
                            "three_stage_technique" => 20.0,
                            "flame_blade" => 6.0,
                            "fire_dash" => 10.0,
                            "flame_assault" => 18.0,
                            "matchlock_gun" => 25.0,
                            _ => 5.0, // 默認冷卻時間
                        };
                        debug!("應用本地技能施放: {} (冷卻 {:.1}s)", ability_id, ability.cooldown_remaining);
                    }
                }
            },
            "use_item" => {
                if let Some(item_id) = result.get("item_id").and_then(|v| v.as_str()) {
                    if let Some(item) = self.local_player.items.iter_mut()
                        .find(|i| i.item_id == item_id) {
                        if item.charges > 0 {
                            item.charges -= 1;
                            item.last_used = Some(SystemTime::now());
                            // 設置測試冷卻時間
                            item.cooldown_remaining = match item_id {
                                "health_potion" => 3.0,
                                "mana_potion" => 2.0,
                                "teleport_scroll" => 60.0,
                                "smoke_bomb" => 15.0,
                                _ => 5.0,
                            };
                            item.is_available = false;
                            debug!("使用道具: {} (剩餘 {} 個，冷卻 {:.1}s)", item_id, item.charges, item.cooldown_remaining);
                        }
                    }
                }
            },
            _ => {
                debug!("應用本地操作: {}", action);
            }
        }
        
        self.last_update = SystemTime::now();
    }
    
    /// 獲取可用技能列表
    pub fn get_available_abilities(&self) -> Vec<&AbilityState> {
        self.local_player.abilities.iter()
            .filter(|a| a.is_available)
            .collect()
    }
    
    /// 獲取玩家狀態摘要
    pub fn get_status_summary(&self) -> String {
        format!(
            "玩家: {} ({}) | 位置: ({:.1}, {:.1}) | 生命值: {:.0}/{:.0} | 召喚物: {} | 同步錯誤: {}",
            self.local_player.name,
            self.local_player.hero_type,
            self.local_player.position.x,
            self.local_player.position.y,
            self.local_player.health.0,
            self.local_player.health.1,
            self.local_player.summons.len(),
            self.sync_errors
        )
    }
    
    /// 更新技能和道具冷卻時間（每幀調用）
    pub fn update_cooldowns(&mut self, delta_time: f32) {
        // 更新技能冷卻
        for ability in &mut self.local_player.abilities {
            if ability.cooldown_remaining > 0.0 {
                ability.cooldown_remaining -= delta_time;
                if ability.cooldown_remaining <= 0.0 {
                    ability.cooldown_remaining = 0.0;
                    ability.is_available = true;
                }
            }
        }
        
        // 更新道具冷卻
        for item in &mut self.local_player.items {
            if item.cooldown_remaining > 0.0 {
                item.cooldown_remaining -= delta_time;
                if item.cooldown_remaining <= 0.0 {
                    item.cooldown_remaining = 0.0;
                    item.is_available = true;
                }
            }
        }
    }
    
    /// 檢查是否有有效的遊戲資料
    /// 判斷標準：玩家位置不為零點，或有其他玩家/實體資料
    pub fn has_valid_data(&self) -> bool {
        // 如果玩家位置不在原點，表示已經有位置資料
        if self.local_player.position != Vec2::zero() {
            return true;
        }
        
        // 或者有其他玩家資料
        if !self.other_players.is_empty() {
            return true;
        }
        
        // 或者有遊戲實體資料
        if !self.entities.is_empty() {
            return true;
        }
        
        // 或者有召喚物資料
        if !self.local_player.summons.is_empty() {
            return true;
        }
        
        false
    }
}