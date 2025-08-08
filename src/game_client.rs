/// 遊戲客戶端核心
/// 
/// 模擬真實遊戲客戶端，處理與 omobab 後端的連接和通信
use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use std::time::Duration;
use tokio::time::sleep;
use log::{info, warn, error, debug};
use anyhow::Result;

use crate::mqtt_handler::MqttHandler;
use crate::game_state::GameState;
use crate::player::PlayerSimulator;

/// 遊戲客戶端配置
#[derive(Debug, Clone)]
pub struct GameClientConfig {
    pub server_ip: String,
    pub server_port: u16,
    pub client_id: String,
    pub player_name: String,
    pub hero_type: String,
}

impl Default for GameClientConfig {
    fn default() -> Self {
        Self {
            server_ip: "127.0.0.1".to_string(),
            server_port: 1883,
            client_id: "omobaf_player".to_string(),
            player_name: "TestPlayer".to_string(),
            hero_type: "saika_magoichi".to_string(),
        }
    }
}

/// 遊戲客戶端狀態
#[derive(Debug, Clone, PartialEq)]
pub enum ClientState {
    Disconnected,
    Connecting,
    Connected,
    InGame,
    Error(String),
}

/// 遊戲客戶端
pub struct GameClient {
    config: GameClientConfig,
    state: ClientState,
    mqtt_handler: MqttHandler,
    game_state: GameState,
    player_simulator: PlayerSimulator,
    client: Option<AsyncClient>,
    shared_game_state: Option<std::sync::Arc<tokio::sync::Mutex<GameState>>>,
    screen_request_handle: Option<tokio::task::JoinHandle<()>>,
}

impl GameClient {
    /// 創建新的遊戲客戶端
    pub fn new(config: GameClientConfig) -> Self {
        let mqtt_handler = MqttHandler::new();
        let game_state = GameState::new(config.player_name.clone(), config.hero_type.clone());
        let player_simulator = PlayerSimulator::new(config.player_name.clone(), config.hero_type.clone());
        
        info!("遊戲客戶端已創建 - 玩家: {}, 英雄: {}", config.player_name, config.hero_type);
        
        Self {
            config,
            state: ClientState::Disconnected,
            mqtt_handler,
            game_state,
            player_simulator,
            client: None,
            shared_game_state: None,
            screen_request_handle: None,
        }
    }
    
    /// 連接到遊戲服務器
    pub async fn connect(&mut self) -> Result<()> {
        info!("正在連接到遊戲服務器 {}:{}", self.config.server_ip, self.config.server_port);
        self.state = ClientState::Connecting;
        
        let mut mqttoptions = MqttOptions::new(
            &self.config.client_id,
            &self.config.server_ip,
            self.config.server_port
        );
        mqttoptions.set_keep_alive(Duration::from_secs(30));
        mqttoptions.set_clean_session(true);
        
        let (client, mut connection) = AsyncClient::new(mqttoptions, 10);
        self.client = Some(client.clone());
        
        // 訂閱遊戲相關主題
        self.subscribe_game_topics(&client).await?;
        
        // 啟動 MQTT 事件處理循環 - 使用 Arc<Mutex> 來共享遊戲狀態
        let mqtt_handler = self.mqtt_handler.clone();
        let game_state = std::sync::Arc::new(tokio::sync::Mutex::new(self.game_state.clone()));
        let game_state_clone = game_state.clone();
        
        // 保存共享的遊戲狀態引用以供後續使用
        self.shared_game_state = Some(game_state);
        
        // 啟動 MQTT 事件處理循環
        tokio::spawn(async move {
            loop {
                match connection.poll().await {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        let mut state = game_state_clone.lock().await;
                        if let Err(e) = mqtt_handler.handle_message(&publish, &mut *state).await {
                            error!("處理 MQTT 訊息失敗: {}", e);
                        } else {
                            debug!("MQTT 訊息處理成功 - 主題: {}", publish.topic);
                        }
                    },
                    Ok(_) => {},
                    Err(e) => {
                        error!("MQTT 連接錯誤: {}", e);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });
        
        
        self.state = ClientState::Connected;
        info!("已成功連接到遊戲服務器");
        
        Ok(())
    }
    
    /// 訂閱遊戲相關主題
    async fn subscribe_game_topics(&self, client: &AsyncClient) -> Result<()> {
        // 訂閱遊戲狀態主題 (實際後端使用的主題)
        client.subscribe("td/all/res", QoS::AtLeastOnce).await?;
        debug!("已訂閱遊戲狀態主題: td/all/res");
        
        // 也訂閱玩家特定主題
        client.subscribe("td/+/send", QoS::AtLeastOnce).await?;
        debug!("已訂閱玩家主題: td/+/send");
        
        // 訂閱畫面狀態回應主題 (使用 player_name 而不是 client_id)
        let screen_response_topic = format!("td/{}/screen_response", self.config.player_name);
        client.subscribe(&screen_response_topic, QoS::AtLeastOnce).await?;
        debug!("已訂閱畫面狀態回應主題: {}", screen_response_topic);
        
        // 訂閱能力測試主題（如果需要）
        client.subscribe("ability_test/response", QoS::AtMostOnce).await?;
        debug!("已訂閱能力測試回應主題");
        
        Ok(())
    }
    
    /// 進入遊戲
    pub async fn enter_game(&mut self) -> Result<()> {
        if self.state != ClientState::Connected {
            return Err(anyhow::anyhow!("客戶端未連接到服務器"));
        }
        
        info!("進入遊戲 - 玩家: {}, 英雄: {}", self.config.player_name, self.config.hero_type);
        
        // 計算視野範圍（每個字符代表10x10單位，終端大小決定總視野）
        const WORLD_UNITS_PER_CHAR: f32 = 10.0;
        let (term_width, term_height) = crossterm::terminal::size().unwrap_or((80, 24));
        let view_width = term_width as f32 * WORLD_UNITS_PER_CHAR;
        let view_height = term_height as f32 * WORLD_UNITS_PER_CHAR;
        
        // 發送進入遊戲訊息，包含視野範圍
        self.send_player_action("enter_game", serde_json::json!({
            "player_name": self.config.player_name,
            "hero_type": self.config.hero_type,
            "viewport": {
                "width": view_width,
                "height": view_height,
                "units_per_char": WORLD_UNITS_PER_CHAR
            }
        })).await?;
        
        // 更新本地視野設定
        self.game_state.viewport.width = view_width;
        self.game_state.viewport.height = view_height;
        
        self.state = ClientState::InGame;
        info!("已進入遊戲");
        
        // 發送初始視窗範圍
        self.send_viewport_update().await?;
        
        // 啟動定期畫面狀態請求循環
        self.start_screen_request_loop().await?;
        
        Ok(())
    }
    
    /// 執行玩家操作
    pub async fn perform_action(&mut self, action: &str, params: serde_json::Value) -> Result<()> {
        if self.state != ClientState::InGame {
            return Err(anyhow::anyhow!("玩家未在遊戲中"));
        }
        
        debug!("執行玩家操作: {} - 參數: {}", action, params);
        
        // 通過模擬器處理操作
        let result = self.player_simulator.perform_action(action, params.clone()).await?;
        
        // 發送操作到服務器
        self.send_player_action(action, params.clone()).await?;
        
        // 更新本地遊戲狀態
        self.game_state.apply_local_action(action, &result);
        
        // 如果是移動操作，發送視野範圍更新
        if action == "move" {
            self.send_viewport_update().await?;
        }
        
        Ok(())
    }
    
    /// 發送視窗範圍更新
    pub async fn send_viewport_update(&self) -> Result<()> {
        // 使用玩家當前位置作為視野中心
        let player_pos = self.game_state.local_player.position;
        
        // 計算視野邊界（考慮每個字符代表10x10單位）
        const WORLD_UNITS_PER_CHAR: f32 = 10.0;
        let (term_width, term_height) = crossterm::terminal::size().unwrap_or((80, 24));
        let view_width = term_width as f32 * WORLD_UNITS_PER_CHAR;
        let view_height = term_height as f32 * WORLD_UNITS_PER_CHAR;
        
        let min_x = player_pos.x - view_width / 2.0;
        let min_y = player_pos.y - view_height / 2.0;
        let max_x = player_pos.x + view_width / 2.0;
        let max_y = player_pos.y + view_height / 2.0;
        
        let viewport_data = serde_json::json!({
            "center_x": player_pos.x,
            "center_y": player_pos.y,
            "width": view_width,
            "height": view_height,
            "units_per_char": WORLD_UNITS_PER_CHAR,
            "min_x": min_x,
            "min_y": min_y,
            "max_x": max_x,
            "max_y": max_y,
        });
        
        debug!("發送視野更新: 中心({:.1}, {:.1}), 範圍({:.1}x{:.1})", 
               player_pos.x, player_pos.y, view_width, view_height);
        
        self.send_player_action("update_viewport", viewport_data).await?;
        debug!("已發送視窗範圍更新");
        
        Ok(())
    }
    
    /// 發送玩家操作到服務器
    async fn send_player_action(&self, action: &str, data: serde_json::Value) -> Result<()> {
        if let Some(client) = &self.client {
            let topic = format!("td/{}/action", self.config.player_name);
            let message = serde_json::json!({
                "t": "player_action",
                "a": action,
                "d": data
            });
            
            client.publish(
                &topic,
                QoS::AtLeastOnce,
                false,
                message.to_string()
            ).await?;
            
            debug!("已發送玩家操作: {} 到主題: {}", action, topic);
        }
        
        Ok(())
    }
    
    /// 自動遊戲模式
    pub async fn auto_play(&mut self, duration_secs: u64) -> Result<()> {
        if self.state != ClientState::InGame {
            return Err(anyhow::anyhow!("玩家未在遊戲中"));
        }
        
        info!("開始自動遊戲模式，持續 {} 秒", duration_secs);
        
        let end_time = std::time::Instant::now() + Duration::from_secs(duration_secs);
        
        while std::time::Instant::now() < end_time {
            // 生成隨機操作
            if let Some((action, params)) = self.player_simulator.generate_random_action() {
                if let Err(e) = self.perform_action(&action, params).await {
                    warn!("自動操作失敗: {}", e);
                }
            }
            
            // 等待一段時間後執行下一個操作
            sleep(Duration::from_millis(1000)).await;
        }
        
        info!("自動遊戲模式結束");
        Ok(())
    }
    
    /// 獲取客戶端狀態
    pub fn get_state(&self) -> &ClientState {
        &self.state
    }
    
    /// 獲取遊戲狀態
    pub fn get_game_state(&self) -> &GameState {
        &self.game_state
    }
    
    /// 獲取可變遊戲狀態
    pub fn get_game_state_mut(&mut self) -> &mut GameState {
        &mut self.game_state
    }
    
    /// 同步共享遊戲狀態
    pub async fn sync_shared_state(&mut self) -> Result<()> {
        if let Some(shared_state) = &self.shared_game_state {
            let state = shared_state.lock().await;
            self.game_state = state.clone();
            debug!("同步共享遊戲狀態完成");
        }
        Ok(())
    }
    
    /// 發送固定範圍畫面請求
    pub async fn request_screen_area(&self, min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Result<()> {
        if let Some(client) = &self.client {
            let request_message = serde_json::json!({
                "t": "screen_request",
                "a": "get_screen_area",
                "d": {
                    "player_name": self.config.player_name,
                    "request_type": "fixed_area",
                    "min_x": min_x,
                    "min_y": min_y,
                    "max_x": max_x,
                    "max_y": max_y,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                }
            });
            
            let topic = format!("td/{}/request", self.config.player_name);
            client.publish(
                &topic,
                QoS::AtLeastOnce,
                false,
                request_message.to_string()
            ).await?;
            
            info!("🔄 已發送固定範圍畫面請求: ({},{}) 到 ({},{}) 到主題: {}", 
                  min_x, min_y, max_x, max_y, topic);
        }
        Ok(())
    }

    /// 啟動畫面狀態請求循環
    async fn start_screen_request_loop(&mut self) -> Result<()> {
        if let Some(client) = &self.client {
            let client_for_requests = client.clone();
            let player_name = self.config.player_name.clone();
            
            info!("🔄 啟動畫面狀態請求循環 (每3秒一次)");
            
            let handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(3));
                loop {
                    interval.tick().await;
                    
                    // 發送畫面狀態請求 - 以玩家為中心的範圍請求
                    let request_message = serde_json::json!({
                        "name": player_name,  // 添加 name 欄位
                        "t": "screen_request",
                        "a": "get_screen_area", 
                        "d": {
                            "player_name": player_name,
                            "request_type": "player_centered",
                            "center_x": 0.0,  // 將由後端根據玩家位置計算
                            "center_y": 0.0,
                            "width": 120.0,   // ±60 範圍
                            "height": 80.0,   // ±40 範圍
                            "timestamp": std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis()
                        }
                    });
                    
                    let topic = format!("td/{}/send", player_name);
                    if let Err(e) = client_for_requests.publish(
                        &topic,
                        QoS::AtLeastOnce,
                        false,
                        request_message.to_string()
                    ).await {
                        warn!("發送畫面狀態請求失敗: {}", e);
                    } else {
                        info!("🔄 已發送畫面狀態請求到主題: {}", topic);
                    }
                }
            });
            
            self.screen_request_handle = Some(handle);
        }
        
        Ok(())
    }
    
    /// 斷開連接
    pub async fn disconnect(&mut self) -> Result<()> {
        // 停止畫面請求循環
        if let Some(handle) = self.screen_request_handle.take() {
            handle.abort();
            info!("已停止畫面狀態請求循環");
        }
        
        if let Some(client) = &self.client {
            // 發送離開遊戲訊息
            if self.state == ClientState::InGame {
                let _ = self.send_player_action("leave_game", serde_json::json!({})).await;
            }
            
            client.disconnect().await?;
        }
        
        self.state = ClientState::Disconnected;
        self.client = None;
        
        info!("已斷開與遊戲服務器的連接");
        Ok(())
    }
}