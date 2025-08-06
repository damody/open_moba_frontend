/// 玩家操作模擬器
/// 
/// 模擬真實玩家的遊戲操作行為
use serde::{Deserialize, Serialize};
use serde_json;
use rand::{thread_rng, Rng};
use log::{info, debug};
use anyhow::Result;
use vek::Vec2;

/// 玩家操作模擬器
#[derive(Debug, Clone)]
pub struct PlayerSimulator {
    pub player_name: String,
    pub hero_type: String,
    pub current_position: Vec2<f32>,
    pub action_history: Vec<PlayerAction>,
    pub auto_mode_enabled: bool,
}

/// 玩家操作記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    pub action_type: String,
    pub timestamp: std::time::SystemTime,
    pub parameters: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

/// 移動參數
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveParams {
    pub target_x: f32,
    pub target_y: f32,
    pub speed: Option<f32>,
}

/// 技能參數
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastAbilityParams {
    pub ability_id: String,
    pub target_position: Option<(f32, f32)>,
    pub target_entity: Option<u32>,
    pub level: Option<u8>,
}

/// 攻擊參數
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackParams {
    pub target_position: (f32, f32),
    pub attack_type: String,  // "basic", "ability", "ranged"
}

impl PlayerSimulator {
    /// 創建新的玩家模擬器
    pub fn new(player_name: String, hero_type: String) -> Self {
        info!("創建玩家模擬器 - 玩家: {}, 英雄: {}", player_name, hero_type);
        
        Self {
            player_name,
            hero_type,
            current_position: Vec2::new(400.0, 300.0), // 預設起始位置
            action_history: Vec::new(),
            auto_mode_enabled: false,
        }
    }
    
    /// 執行玩家操作
    pub async fn perform_action(&mut self, action: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        debug!("執行玩家操作: {} - 參數: {}", action, params);
        
        let result = match action {
            "move" => self.handle_move_action(params.clone()).await?,
            "cast_ability" => self.handle_cast_ability_action(params.clone()).await?,
            "attack" => self.handle_attack_action(params.clone()).await?,
            "interact" => self.handle_interact_action(params.clone()).await?,
            _ => {
                return Err(anyhow::anyhow!("未知的操作類型: {}", action));
            }
        };
        
        // 記錄操作歷史
        let action_record = PlayerAction {
            action_type: action.to_string(),
            timestamp: std::time::SystemTime::now(),
            parameters: params,
            result: Some(result.clone()),
        };
        
        self.action_history.push(action_record);
        
        // 限制歷史記錄長度
        if self.action_history.len() > 100 {
            self.action_history.remove(0);
        }
        
        Ok(result)
    }
    
    /// 處理移動操作
    async fn handle_move_action(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let move_params: MoveParams = serde_json::from_value(params)?;
        
        // 計算移動距離和有效性
        let target_pos = Vec2::new(move_params.target_x, move_params.target_y);
        let distance = (target_pos - self.current_position).magnitude();
        
        // 限制最大移動距離（防止瞬移）
        let max_move_distance = 200.0;
        let actual_target = if distance > max_move_distance {
            let direction = (target_pos - self.current_position).normalized();
            self.current_position + direction * max_move_distance
        } else {
            target_pos
        };
        
        // 更新位置
        self.current_position = actual_target;
        
        debug!("玩家 {} 移動到位置: ({:.1}, {:.1})", 
               self.player_name, actual_target.x, actual_target.y);
        
        Ok(serde_json::json!({
            "x": actual_target.x,
            "y": actual_target.y,
            "distance_moved": distance.min(max_move_distance),
            "success": true
        }))
    }
    
    /// 處理技能施放操作
    async fn handle_cast_ability_action(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let cast_params: CastAbilityParams = serde_json::from_value(params)?;
        
        // 驗證技能是否屬於當前英雄
        if !self.is_ability_valid(&cast_params.ability_id) {
            return Err(anyhow::anyhow!("技能 {} 不屬於英雄 {}", cast_params.ability_id, self.hero_type));
        }
        
        // 計算施法位置
        let cast_position = cast_params.target_position
            .unwrap_or((self.current_position.x, self.current_position.y));
        
        debug!("玩家 {} 施放技能: {} 在位置 ({:.1}, {:.1})", 
               self.player_name, cast_params.ability_id, cast_position.0, cast_position.1);
        
        Ok(serde_json::json!({
            "ability_id": cast_params.ability_id,
            "level": cast_params.level.unwrap_or(1),
            "cast_position": cast_position,
            "target_entity": cast_params.target_entity,
            "success": true
        }))
    }
    
    /// 處理攻擊操作
    async fn handle_attack_action(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        let attack_params: AttackParams = serde_json::from_value(params)?;
        
        let target_distance = {
            let target_pos = Vec2::new(attack_params.target_position.0, attack_params.target_position.1);
            (target_pos - self.current_position).magnitude()
        };
        
        // 檢查攻擊範圍
        let max_attack_range = match attack_params.attack_type.as_str() {
            "basic" => 50.0,
            "ranged" => 150.0,
            "ability" => 200.0,
            _ => 50.0,
        };
        
        let can_attack = target_distance <= max_attack_range;
        
        debug!("玩家 {} 攻擊位置 ({:.1}, {:.1}) - 距離: {:.1}, 可攻擊: {}", 
               self.player_name, attack_params.target_position.0, attack_params.target_position.1, 
               target_distance, can_attack);
        
        Ok(serde_json::json!({
            "target_position": attack_params.target_position,
            "attack_type": attack_params.attack_type,
            "distance": target_distance,
            "in_range": can_attack,
            "success": can_attack
        }))
    }
    
    /// 處理互動操作
    async fn handle_interact_action(&mut self, params: serde_json::Value) -> Result<serde_json::Value> {
        debug!("玩家 {} 執行互動操作: {}", self.player_name, params);
        
        Ok(serde_json::json!({
            "interaction_type": params.get("type").unwrap_or(&serde_json::Value::String("generic".to_string())),
            "success": true
        }))
    }
    
    /// 生成隨機操作（自動遊戲模式）
    pub fn generate_random_action(&self) -> Option<(String, serde_json::Value)> {
        if !self.auto_mode_enabled {
            return None;
        }
        
        let mut rng = thread_rng();
        let action_type = rng.gen_range(0..4);
        
        match action_type {
            0 => {
                // 隨機移動
                let target_x = self.current_position.x + rng.gen_range(-100.0..100.0);
                let target_y = self.current_position.y + rng.gen_range(-100.0..100.0);
                
                Some(("move".to_string(), serde_json::json!({
                    "target_x": target_x.max(0.0).min(800.0),
                    "target_y": target_y.max(0.0).min(600.0)
                })))
            },
            1 => {
                // 隨機施放技能
                let abilities = self.get_hero_abilities();
                if !abilities.is_empty() {
                    let ability = &abilities[rng.gen_range(0..abilities.len())];
                    
                    Some(("cast_ability".to_string(), serde_json::json!({
                        "ability_id": ability,
                        "target_position": [
                            self.current_position.x + rng.gen_range(-50.0..50.0),
                            self.current_position.y + rng.gen_range(-50.0..50.0)
                        ],
                        "level": 1
                    })))
                } else {
                    None
                }
            },
            2 => {
                // 隨機攻擊
                let target_x = self.current_position.x + rng.gen_range(-80.0..80.0);
                let target_y = self.current_position.y + rng.gen_range(-80.0..80.0);
                
                Some(("attack".to_string(), serde_json::json!({
                    "target_position": [target_x, target_y],
                    "attack_type": "basic"
                })))
            },
            _ => {
                // 等待（無操作）
                None
            }
        }
    }
    
    /// 驗證技能是否有效
    fn is_ability_valid(&self, ability_id: &str) -> bool {
        let hero_abilities = self.get_hero_abilities();
        hero_abilities.contains(&ability_id.to_string())
    }
    
    /// 獲取英雄技能列表
    fn get_hero_abilities(&self) -> Vec<String> {
        match self.hero_type.as_str() {
            "saika_magoichi" => vec![
                "sniper_mode".to_string(),
                "saika_reinforcements".to_string(),
                "rain_iron_cannon".to_string(),
                "three_stage_technique".to_string(),
            ],
            "date_masamune" => vec![
                "flame_blade".to_string(),
                "fire_dash".to_string(),
                "flame_assault".to_string(),
                "matchlock_gun".to_string(),
            ],
            _ => vec![]
        }
    }
    
    /// 設置自動模式
    pub fn set_auto_mode(&mut self, enabled: bool) {
        self.auto_mode_enabled = enabled;
        info!("玩家 {} 自動模式: {}", self.player_name, if enabled { "開啟" } else { "關閉" });
    }
    
    /// 獲取操作歷史統計
    pub fn get_action_stats(&self) -> serde_json::Value {
        let mut stats = std::collections::HashMap::new();
        
        for action in &self.action_history {
            *stats.entry(action.action_type.clone()).or_insert(0) += 1;
        }
        
        serde_json::json!({
            "total_actions": self.action_history.len(),
            "action_counts": stats,
            "current_position": [self.current_position.x, self.current_position.y],
            "auto_mode": self.auto_mode_enabled
        })
    }
    
    /// 執行預設操作序列（用於演示）
    pub fn get_demo_sequence(&self) -> Vec<(String, serde_json::Value)> {
        let abilities = self.get_hero_abilities();
        let mut sequence = Vec::new();
        
        // 移動序列
        sequence.push(("move".to_string(), serde_json::json!({
            "target_x": 300.0,
            "target_y": 200.0
        })));
        
        // 施放第一個技能
        if !abilities.is_empty() {
            sequence.push(("cast_ability".to_string(), serde_json::json!({
                "ability_id": abilities[0],
                "target_position": [350.0, 250.0],
                "level": 1
            })));
        }
        
        // 攻擊
        sequence.push(("attack".to_string(), serde_json::json!({
            "target_position": [400.0, 300.0],
            "attack_type": "basic"
        })));
        
        // 移動到新位置
        sequence.push(("move".to_string(), serde_json::json!({
            "target_x": 500.0,
            "target_y": 400.0
        })));
        
        // 如果有召喚技能，施放它
        if abilities.contains(&"saika_reinforcements".to_string()) {
            sequence.push(("cast_ability".to_string(), serde_json::json!({
                "ability_id": "saika_reinforcements",
                "target_position": [450.0, 350.0],
                "level": 1
            })));
        }
        
        sequence
    }
}