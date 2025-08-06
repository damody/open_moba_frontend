/// MQTT 訊息處理器
/// 
/// 處理來自 omobab 後端的遊戲 MQTT 訊息
use rumqttc::Publish;
use serde::{Deserialize, Serialize};
use serde_json;
use log::{info, warn, debug, error};
use anyhow::Result;
use std::time::SystemTime;

use crate::game_state::GameState;

/// MQTT 訊息格式（對應後端的 MqttMsg）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MqttMessage {
    pub topic: String,
    pub msg: String,
    pub time: SystemTime,
}

/// 玩家數據格式（對應後端的 PlayerData）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerData {
    pub name: String,
    pub t: String,      // 類型
    pub a: String,      // 動作
    pub d: serde_json::Value,  // 數據
}

/// MQTT 訊息處理器
#[derive(Debug, Clone)]
pub struct MqttHandler {
    // 統計信息
    pub messages_received: u64,
    pub messages_processed: u64,
    pub last_message_time: Option<SystemTime>,
}

impl MqttHandler {
    /// 創建新的 MQTT 處理器
    pub fn new() -> Self {
        Self {
            messages_received: 0,
            messages_processed: 0,
            last_message_time: None,
        }
    }
    
    /// 處理接收到的 MQTT 訊息
    pub async fn handle_message(&self, publish: &Publish, game_state: &mut GameState) -> Result<()> {
        let mut handler = self.clone();
        handler.messages_received += 1;
        handler.last_message_time = Some(SystemTime::now());
        
        let topic = &publish.topic;
        let payload = String::from_utf8_lossy(&publish.payload);
        
        // 增強調試信息 - 顯示收到的消息
        info!("📨 收到 MQTT 訊息 - 主題: {}, 負載: {}", topic, payload);
        
        // 根據主題路由訊息
        match self.route_message(topic, &payload, game_state).await {
            Ok(_) => {
                handler.messages_processed += 1;
                info!("✅ MQTT 訊息處理成功 - 主題: {}", topic);
            },
            Err(e) => {
                warn!("❌ MQTT 訊息處理失敗 - 主題: {}, 錯誤: {}", topic, e);
            }
        }
        
        Ok(())
    }
    
    /// 根據主題路由訊息
    async fn route_message(&self, topic: &str, payload: &str, game_state: &mut GameState) -> Result<()> {
        if topic == "td/all/res" {
            // 後端遊戲狀態廣播訊息
            self.handle_game_broadcast_message(topic, payload, game_state).await
        } else if topic.starts_with("td/") && topic.ends_with("/send") {
            // 玩家特定遊戲狀態訊息
            self.handle_game_state_message(topic, payload, game_state).await
        } else if topic.starts_with("td/") && topic.ends_with("/screen_response") {
            // 畫面狀態回應訊息
            self.handle_screen_response_message(topic, payload, game_state).await
        } else if topic == "ability_test/response" {
            // 能力測試回應
            self.handle_ability_test_response(payload, game_state).await
        } else {
            debug!("未知主題: {}", topic);
            Ok(())
        }
    }
    
    /// 處理遊戲廣播訊息 (td/all/res)
    async fn handle_game_broadcast_message(&self, topic: &str, payload: &str, game_state: &mut GameState) -> Result<()> {
        info!("收到遊戲廣播訊息 - 主題: {}, 負載: {}", topic, payload);
        
        // 嘗試解析 PlayerData 格式
        match serde_json::from_str::<PlayerData>(payload) {
            Ok(player_data) => {
                info!("解析廣播數據 - 類型: {}, 動作: {}", player_data.t, player_data.a);
                self.process_broadcast_data(&player_data, game_state).await
            },
            Err(_) => {
                // 如果不是 PlayerData 格式，嘗試直接解析 JSON
                match serde_json::from_str::<serde_json::Value>(payload) {
                    Ok(data) => {
                        info!("解析原始廣播數據: {}", data);
                        self.process_raw_game_data(&data, game_state).await
                    },
                    Err(e) => {
                        warn!("無法解析廣播訊息: {}", e);
                        Ok(())
                    }
                }
            }
        }
    }
    
    /// 處理廣播數據
    async fn process_broadcast_data(&self, player_data: &PlayerData, game_state: &mut GameState) -> Result<()> {
        info!("處理廣播數據 - 類型: {}, 動作: {}", player_data.t, player_data.a);
        
        match player_data.t.as_str() {
            "creep" => {
                info!("收到 creep 廣播: {}", player_data.d);
                // 處理小兵相關訊息
            },
            "tower" => {
                info!("收到 tower 廣播: {}", player_data.d);
                // 處理塔相關訊息
            },
            "player" => {
                info!("收到 player 廣播: {}", player_data.d);
                // 處理玩家相關訊息
            },
            "projectile" => {
                info!("收到 projectile 廣播: {}", player_data.d);
                // 處理投射物相關訊息
            },
            _ => {
                debug!("未知的廣播數據類型: {}", player_data.t);
            }
        }
        
        Ok(())
    }

    /// 處理遊戲狀態訊息 (td/+/send)
    async fn handle_game_state_message(&self, topic: &str, payload: &str, game_state: &mut GameState) -> Result<()> {
        // 解析主題以獲取玩家名稱
        let parts: Vec<&str> = topic.split('/').collect();
        if parts.len() >= 2 {
            let player_name = parts[1];
            debug!("處理玩家 {} 的遊戲狀態更新", player_name);
        }
        
        // 嘗試解析 PlayerData 格式
        match serde_json::from_str::<PlayerData>(payload) {
            Ok(player_data) => {
                self.process_player_data(&player_data, game_state).await
            },
            Err(_) => {
                // 如果不是 PlayerData 格式，嘗試直接解析 JSON
                match serde_json::from_str::<serde_json::Value>(payload) {
                    Ok(data) => {
                        self.process_raw_game_data(&data, game_state).await
                    },
                    Err(e) => {
                        warn!("無法解析遊戲狀態訊息: {}", e);
                        Ok(())
                    }
                }
            }
        }
    }
    
    /// 處理 PlayerData 格式的訊息
    async fn process_player_data(&self, player_data: &PlayerData, game_state: &mut GameState) -> Result<()> {
        debug!("處理玩家數據 - 玩家: {}, 類型: {}, 動作: {}", 
               player_data.name, player_data.t, player_data.a);
        
        match player_data.t.as_str() {
            "position" => {
                // 位置更新
                if let Ok(pos_data) = serde_json::from_value::<PositionData>(player_data.d.clone()) {
                    game_state.update_player_position(&player_data.name, pos_data.x, pos_data.y);
                    debug!("更新玩家 {} 位置: ({}, {})", player_data.name, pos_data.x, pos_data.y);
                }
            },
            "ability" => {
                // 技能使用
                if let Ok(ability_data) = serde_json::from_value::<AbilityData>(player_data.d.clone()) {
                    game_state.update_player_ability(&player_data.name, &ability_data);
                    debug!("玩家 {} 使用技能: {}", player_data.name, ability_data.ability_id);
                }
            },
            "health" => {
                // 生命值更新
                if let Ok(health_data) = serde_json::from_value::<HealthData>(player_data.d.clone()) {
                    game_state.update_player_health(&player_data.name, health_data.current, health_data.max);
                    debug!("更新玩家 {} 生命值: {}/{}", player_data.name, health_data.current, health_data.max);
                }
            },
            "summon" => {
                // 召喚物更新
                if let Ok(summon_data) = serde_json::from_value::<SummonData>(player_data.d.clone()) {
                    game_state.update_summon_state(&player_data.name, &summon_data);
                    debug!("玩家 {} 召喚物更新: {}", player_data.name, summon_data.unit_type);
                }
            },
            _ => {
                debug!("未知的玩家數據類型: {}", player_data.t);
            }
        }
        
        Ok(())
    }
    
    /// 處理原始遊戲數據
    async fn process_raw_game_data(&self, data: &serde_json::Value, game_state: &mut GameState) -> Result<()> {
        debug!("處理原始遊戲數據: {}", data);
        
        // 嘗試提取常見的遊戲狀態字段
        if let Some(players) = data.get("players") {
            if let Ok(player_states) = serde_json::from_value::<Vec<PlayerState>>(players.clone()) {
                for player_state in player_states {
                    game_state.sync_player_state(&player_state);
                }
            }
        }
        
        if let Some(_entities) = data.get("entities") {
            // 處理實體狀態更新
            debug!("收到實體狀態更新");
        }
        
        Ok(())
    }
    
    /// 處理畫面狀態回應訊息
    async fn handle_screen_response_message(&self, topic: &str, payload: &str, game_state: &mut GameState) -> Result<()> {
        info!("🖥️ 收到畫面狀態回應 - 主題: {}", topic);
        info!("📄 Screen response payload (前100字符): {}", &payload[..std::cmp::min(100, payload.len())]);
        debug!("畫面狀態回應內容: {}", payload);
        
        match serde_json::from_str::<ScreenResponse>(payload) {
            Ok(response) => {
                info!("解析畫面狀態回應成功 - 範圍: {:?}", response.d.area);
                
                // 更新視口範圍
                if let Some(area) = &response.d.area {
                    game_state.viewport.center.x = (area.min_x + area.max_x) / 2.0;
                    game_state.viewport.center.y = (area.min_y + area.max_y) / 2.0;
                    game_state.viewport.width = area.max_x - area.min_x;
                    game_state.viewport.height = area.max_y - area.min_y;
                    debug!("更新視口中心: ({:.1}, {:.1}), 大小: {:.1}x{:.1}", 
                           game_state.viewport.center.x, game_state.viewport.center.y,
                           game_state.viewport.width, game_state.viewport.height);
                }
                
                // 處理實體數據 - 將網路實體轉換為本地實體
                if let Some(entities) = &response.d.entities {
                    for net_entity in entities {
                        // 將網路實體轉換為本地實體格式
                        let entity = crate::game_state::Entity {
                            id: net_entity.id,
                            entity_type: match net_entity.entity_type.as_str() {
                                "player" => crate::game_state::EntityType::Player("unknown".to_string()),
                                "summon" => crate::game_state::EntityType::Summon(net_entity.entity_type.clone()),
                                "projectile" => crate::game_state::EntityType::Projectile,
                                _ => crate::game_state::EntityType::Effect,
                            },
                            position: vek::Vec2::new(net_entity.position.0, net_entity.position.1),
                            health: net_entity.health.unwrap_or((100.0, 100.0)),
                            owner: None,
                        };
                        game_state.entities.insert(entity.id, entity);
                    }
                    info!("更新 {} 個實體", entities.len());
                }
                
                // 處理玩家數據
                if let Some(players) = &response.d.players {
                    for player in players {
                        game_state.other_players.insert(player.name.clone(), player.clone());
                    }
                    info!("更新 {} 個玩家狀態", players.len());
                }
                
                // 更新最後更新時間
                game_state.last_update = SystemTime::now();
                
            },
            Err(e) => {
                warn!("❌ 無法解析畫面狀態回應: {}", e);
                info!("🔍 嘗試解析為原始 JSON...");
                // 嘗試解析為簡單 JSON 對象
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(payload) {
                    info!("✅ 收到原始畫面數據: {}", data);
                } else {
                    error!("❌ 完全無法解析 JSON 數據");
                }
            }
        }
        
        Ok(())
    }

    /// 處理能力測試回應
    async fn handle_ability_test_response(&self, payload: &str, _game_state: &mut GameState) -> Result<()> {
        match serde_json::from_str::<TestResponse>(payload) {
            Ok(response) => {
                info!("收到能力測試回應 - 命令: {}, 成功: {}", response.command, response.success);
                if !response.success {
                    if let Some(error) = response.data.get("error") {
                        warn!("能力測試失敗: {}", error);
                    }
                }
            },
            Err(e) => {
                warn!("無法解析能力測試回應: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// 獲取統計信息
    pub fn get_stats(&self) -> (u64, u64, Option<SystemTime>) {
        (self.messages_received, self.messages_processed, self.last_message_time)
    }
}

/// 位置數據
#[derive(Serialize, Deserialize, Clone, Debug)]
struct PositionData {
    x: f32,
    y: f32,
}

/// 技能數據
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbilityData {
    pub ability_id: String,
    pub level: u8,
    pub cooldown_remaining: f32,
    pub target_position: Option<(f32, f32)>,
    pub target_entity: Option<u32>,
}

/// 生命值數據
#[derive(Serialize, Deserialize, Clone, Debug)]
struct HealthData {
    current: f32,
    max: f32,
}

/// 召喚物數據
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SummonData {
    pub unit_type: String,
    pub position: (f32, f32),
    pub health: f32,
    pub state: String,
}

/// 玩家狀態（完整狀態同步）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerState {
    pub name: String,
    pub hero_type: String,
    pub position: (f32, f32),
    pub health: (f32, f32),  // (current, max)
    pub abilities: Vec<AbilityData>,
    pub summons: Vec<SummonData>,
}

/// 畫面狀態回應格式
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenResponse {
    pub t: String,
    pub d: ScreenData,
}

/// 畫面數據
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenData {
    pub area: Option<ScreenArea>,
    pub entities: Option<Vec<NetworkEntity>>,
    pub players: Option<Vec<PlayerState>>,
    pub projectiles: Option<Vec<ProjectileData>>,
    pub terrain: Option<Vec<TerrainData>>,
    pub timestamp: u64,
}

/// 畫面範圍
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScreenArea {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// 網路實體數據 (用於 MQTT 傳輸)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkEntity {
    pub id: u32,
    pub entity_type: String,
    pub position: (f32, f32),
    pub health: Option<(f32, f32)>,
    pub state: String,
}

/// 投射物數據
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectileData {
    pub id: u32,
    pub projectile_type: String,
    pub position: (f32, f32),
    pub velocity: (f32, f32),
    pub owner: String,
}

/// 地形數據
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TerrainData {
    pub position: (f32, f32),
    pub terrain_type: String,
    pub properties: serde_json::Value,
}

/// 測試回應格式
#[derive(Serialize, Deserialize, Clone, Debug)]
struct TestResponse {
    command: String,
    success: bool,
    data: serde_json::Value,
    timestamp: u64,
    execution_time_ms: u64,
}