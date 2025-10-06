/// CLI 介面
/// 
/// 提供簡潔的命令行操作介面
use clap::{Parser, Subcommand};
use serde_json;
use log::{info, error, warn};
use anyhow::Result;

use crate::game_client::{GameClient, GameClientConfig};
use crate::terminal_view::UserInput;

/// omobaf - Open MOBA Frontend 假遊戲客戶端
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// 服務器 IP 地址
    #[arg(long, default_value = "127.0.0.1")]
    pub server_ip: String,
    
    /// 服務器端口
    #[arg(long, default_value_t = 1883)]
    pub server_port: u16,
    
    /// 客戶端 ID
    #[arg(long, default_value = "omobaf_player")]
    pub client_id: String,
    
    /// 玩家名稱
    #[arg(long, default_value = "TestPlayer")]
    pub player_name: String,
    
    /// 英雄類型
    #[arg(long, default_value = "saika_magoichi")]
    pub hero: String,
    
    /// 詳細日誌輸出
    #[arg(short, long)]
    pub verbose: bool,
}

/// 子命令
#[derive(Subcommand)]
pub enum Commands {
    /// 啟動互動式模式 (可選直接進入 view)
    Interactive {
        /// 自動進入遊戲並啟動實時視圖
        #[arg(long)]
        auto_view: bool,
        /// 視圖大小
        #[arg(short, long, default_value_t = 20.0)]
        size: f32,
        /// 是否顯示視野範圍
        #[arg(long)]
        show_vision: bool,
    },
    
    /// 連接到遊戲服務器
    Connect,
    
    /// 開始遊戲，選擇英雄
    Play {
        /// 英雄類型 (saika_magoichi 或 date_masamune)
        #[arg(short, long)]
        hero: Option<String>,
    },
    
    /// 移動到指定位置
    Move {
        /// X 座標
        x: f32,
        /// Y 座標
        y: f32,
    },
    
    /// 施放技能
    Cast {
        /// 技能 ID
        ability: String,
        /// 目標 X 座標 (可選)
        #[arg(short, long)]
        x: Option<f32>,
        /// 目標 Y 座標 (可選)
        #[arg(short, long)]
        y: Option<f32>,
        /// 技能等級 (可選)
        #[arg(short, long)]
        level: Option<u8>,
    },
    
    /// 攻擊指定位置
    Attack {
        /// 目標 X 座標
        x: f32,
        /// 目標 Y 座標
        y: f32,
        /// 攻擊類型 (basic, ranged, ability)
        #[arg(short, long, default_value = "basic")]
        attack_type: String,
    },
    
    /// 查看遊戲狀態
    Status,
    
    /// 自動遊戲模式
    Auto {
        /// 持續時間（秒）
        #[arg(short, long, default_value_t = 60)]
        duration: u64,
    },
    
    /// 執行演示序列
    Demo,
    
    /// 列出可用技能
    Abilities,
    
    /// 顯示終端視圖
    View {
        /// 視圖範圍半徑（創建正方形視圖）
        #[arg(short, long, conflicts_with_all = ["width", "height"])]
        radius: Option<f32>,
        /// 視圖寬度
        #[arg(short, long)]
        width: Option<f32>,
        /// 視圖高度
        #[arg(short = 'H', long)]
        height: Option<f32>,
        /// 是否顯示視野範圍
        #[arg(long)]
        show_vision: bool,
        /// 是否持續刷新
        #[arg(long, default_value_t = true)]
        live: bool,
    },
    
    /// 斷開連接
    Disconnect,
}

/// CLI 處理器
pub struct CliHandler {
    game_client: Option<GameClient>,
    backend_manager: Option<crate::backend_manager::BackendManager>,
}

impl CliHandler {
    /// 創建新的 CLI 處理器
    pub fn new() -> Self {
        Self {
            game_client: None,
            backend_manager: None,
        }
    }
    
    /// 設置終端日誌系統
    fn setup_terminal_logger(&self, verbose: bool) {
        use log::LevelFilter;
        use std::sync::Arc;
        
        let level = if verbose { LevelFilter::Debug } else { LevelFilter::Info };
        
        let logger = env_logger::Builder::new()
            .filter_level(level)
            .target(env_logger::Target::Pipe(Box::new(crate::terminal_logger::TerminalLogWriter)))
            .build();
            
        if let Err(_) = log::set_boxed_logger(Box::new(logger)) {
            // 日誌系統已經初始化，忽略錯誤
        }
        log::set_max_level(level);
    }
    
    /// 處理 CLI 命令
    pub async fn handle_command(&mut self, cli: Cli) -> Result<()> {
        // 根據命令類型設置不同的日誌系統
        let is_view_command = matches!(cli.command, Commands::View { .. } | Commands::Interactive { auto_view: true, .. });
        
        if is_view_command {
            // 視圖模式使用自定義日誌系統
            self.setup_terminal_logger(cli.verbose);
        } else {
            // 其他模式使用標準日誌系統
            if cli.verbose {
                env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
            } else {
                env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
            }
        }
        
        // 創建遊戲客戶端配置
        let config = GameClientConfig {
            server_ip: cli.server_ip.clone(),
            server_port: cli.server_port,
            client_id: cli.client_id.clone(),
            player_name: cli.player_name.clone(),
            hero_type: cli.hero.clone(),
        };
        
        match cli.command {
            Commands::Interactive { auto_view, size, show_vision } => {
                self.cmd_interactive(config, auto_view, size, show_vision).await
            },
            Commands::Connect => {
                self.cmd_connect(config).await
            },
            Commands::Play { hero } => {
                let hero_type = hero.unwrap_or(cli.hero);
                let mut play_config = config;
                play_config.hero_type = hero_type;
                self.cmd_play(play_config).await
            },
            Commands::Move { x, y } => {
                self.cmd_move(x, y).await
            },
            Commands::Cast { ability, x, y, level } => {
                self.cmd_cast(ability, x, y, level).await
            },
            Commands::Attack { x, y, attack_type } => {
                self.cmd_attack(x, y, attack_type).await
            },
            Commands::Status => {
                self.cmd_status().await
            },
            Commands::Auto { duration } => {
                self.cmd_auto(duration).await
            },
            Commands::Demo => {
                self.cmd_demo().await
            },
            Commands::Abilities => {
                self.cmd_abilities().await
            },
            Commands::View { radius, width, height, show_vision, live } => {
                self.cmd_view(radius, width, height, show_vision, live).await
            },
            Commands::Disconnect => {
                self.cmd_disconnect().await
            },
        }
    }
    
    /// 互動式命令
    async fn cmd_interactive(&mut self, config: GameClientConfig, auto_view: bool, size: f32, show_vision: bool) -> Result<()> {
        if auto_view {
            info!("啟動自動模式：連接 -> 進入遊戲 -> 實時視圖");
            
            // 自動連接和進入遊戲
            self.cmd_connect(config.clone()).await?;
            self.cmd_play(config.clone()).await?;
            
            // 啟動實時視圖
            info!("啟動實時視圖 (大小: {}, 顯示視野: {})", size, show_vision);
            if let Some(client) = &mut self.game_client {
                let view_result = crate::terminal_view::TerminalView::new(size, show_vision);
                match view_result {
                    Ok(mut view) => {
                        info!("啟動實時終端視圖 (按 'q' 或 Esc 退出)");
                        if let Err(e) = view.init_terminal() {
                            error!("初始化終端失敗: {}", e);
                        } else {
                            loop {
                                // 同步共享遊戲狀態
                                if let Err(e) = client.sync_shared_state().await {
                                    error!("同步遊戲狀態失敗: {}", e);
                                }
                                
                                // 更新技能冷卻時間
                                client.get_game_state_mut().update_cooldowns(0.016); // 600ms = 0.6s
                                tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                                match view.render_live(client.get_game_state()) {
                                    Ok(crate::terminal_view::UserInput::Continue) => {
                                        
                                    }
                                    Ok(crate::terminal_view::UserInput::Quit) => break,
                                    Ok(crate::terminal_view::UserInput::Move(world_pos)) => {
                                        info!("移動到: ({:.1}, {:.1})", world_pos.x, world_pos.y);
                                        if let Err(e) = client.perform_action("move", serde_json::json!({
                                            "x": world_pos.x,
                                            "y": world_pos.y
                                        })).await {
                                            error!("移動指令失敗: {}", e);
                                        }
                                    }
                                    Ok(crate::terminal_view::UserInput::Attack(world_pos)) => {
                                        info!("攻擊位置: ({:.1}, {:.1})", world_pos.x, world_pos.y);
                                        if let Err(e) = client.perform_action("attack", serde_json::json!({
                                            "target_position": [world_pos.x, world_pos.y],
                                            "attack_type": "basic"
                                        })).await {
                                            error!("攻擊指令失敗: {}", e);
                                        }
                                    }
                                    Ok(crate::terminal_view::UserInput::MoveAttack(world_pos)) => {
                                        info!("移動攻擊到: ({:.1}, {:.1})", world_pos.x, world_pos.y);
                                        // 先移動再攻擊
                                        if let Err(e) = client.perform_action("move", serde_json::json!({
                                            "x": world_pos.x,
                                            "y": world_pos.y
                                        })).await {
                                            error!("移動攻擊移動部分失敗: {}", e);
                                        } else {
                                            if let Err(e) = client.perform_action("attack", serde_json::json!({
                                                "target_position": [world_pos.x, world_pos.y],
                                                "attack_type": "move_attack"
                                            })).await {
                                                error!("移動攻擊攻擊部分失敗: {}", e);
                                            }
                                        }
                                    }
                                    Ok(crate::terminal_view::UserInput::ForceAttack(world_pos)) => {
                                        info!("強制攻擊位置: ({:.1}, {:.1})", world_pos.x, world_pos.y);
                                        if let Err(e) = client.perform_action("attack", serde_json::json!({
                                            "target_position": [world_pos.x, world_pos.y],
                                            "attack_type": "force_attack"
                                        })).await {
                                            error!("強制攻擊指令失敗: {}", e);
                                        }
                                    }
                                    Ok(crate::terminal_view::UserInput::CastAbility(ability_id, world_pos)) => {
                                        info!("施放技能 {} 於位置: ({:.1}, {:.1})", ability_id, world_pos.x, world_pos.y);
                                        if let Err(e) = client.perform_action("cast_ability", serde_json::json!({
                                            "ability_id": ability_id,
                                            "target_position": [world_pos.x, world_pos.y],
                                            "level": 1
                                        })).await {
                                            error!("技能施放指令失敗: {}", e);
                                        }
                                    }
                                    Ok(crate::terminal_view::UserInput::Cancel) => {
                                        // 技能選擇被取消，繼續遊戲循環
                                    }
                                    Ok(crate::terminal_view::UserInput::UseItem(item_id, _target_pos)) => {
                                        info!("使用道具: {}", item_id);
                                        if let Err(e) = client.perform_action("use_item", serde_json::json!({
                                            "item_id": item_id
                                        })).await {
                                            error!("道具使用指令失敗: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        error!("終端視圖錯誤: {}", e);
                                        break;
                                    }
                                }
                            }
                            let _ = view.cleanup_terminal();
                        }
                    }
                    Err(e) => {
                        error!("創建終端視圖失敗: {}", e);
                    }
                }
            }
        } else {
            // 啟動正常互動式模式
            info!("啟動互動式模式");
            let mut interactive = crate::interactive::InteractiveCli::new();
            interactive.run().await?;
        }
        Ok(())
    }

    /// 連接命令
    async fn cmd_connect(&mut self, config: GameClientConfig) -> Result<()> {
        info!("正在連接到服務器 {}:{}...", config.server_ip, config.server_port);
        
        let mut client = GameClient::new(config);
        info!("🔄 GameClient 已創建，開始連接...");
        
        client.connect().await?;
        info!("✅ GameClient 連接完成");
        
        self.game_client = Some(client);
        info!("連接成功！");
        
        Ok(())
    }
    
    /// 開始遊戲命令
    async fn cmd_play(&mut self, config: GameClientConfig) -> Result<()> {
        info!("開始遊戲 - 玩家: {}, 英雄: {}", config.player_name, config.hero_type);
        
        // 如果沒有連接，先連接
        if self.game_client.is_none() {
            info!("⚠️ 遊戲客戶端未連接，先進行連接...");
            self.cmd_connect(config.clone()).await?;
        }
        
        if let Some(client) = &mut self.game_client {
            info!("🎯 調用 enter_game()...");
            client.enter_game().await?;
            info!("✅ 已進入遊戲！screen_request 循環應該已啟動");
        } else {
            error!("❌ 遊戲客戶端為空，無法進入遊戲");
        }
        
        Ok(())
    }
    
    /// 移動命令
    async fn cmd_move(&mut self, x: f32, y: f32) -> Result<()> {
        if let Some(client) = &mut self.game_client {
            let params = serde_json::json!({
                "target_x": x,
                "target_y": y
            });
            
            client.perform_action("move", params).await?;
            info!("移動到位置 ({}, {})", x, y);
        } else {
            error!("未連接到遊戲服務器。請先使用 'connect' 命令。");
        }
        
        Ok(())
    }
    
    /// 施法命令
    async fn cmd_cast(&mut self, ability: String, x: Option<f32>, y: Option<f32>, level: Option<u8>) -> Result<()> {
        if let Some(client) = &mut self.game_client {
            let mut params = serde_json::json!({
                "ability_id": ability,
                "level": level.unwrap_or(1)
            });
            
            if let (Some(x), Some(y)) = (x, y) {
                params["target_position"] = serde_json::json!([x, y]);
            }
            
            client.perform_action("cast_ability", params).await?;
            info!("施放技能: {}", ability);
        } else {
            error!("未連接到遊戲服務器。請先使用 'connect' 命令。");
        }
        
        Ok(())
    }
    
    /// 攻擊命令
    async fn cmd_attack(&mut self, x: f32, y: f32, attack_type: String) -> Result<()> {
        if let Some(client) = &mut self.game_client {
            let params = serde_json::json!({
                "target_position": [x, y],
                "attack_type": attack_type
            });
            
            client.perform_action("attack", params).await?;
            info!("攻擊位置 ({}, {})", x, y);
        } else {
            error!("未連接到遊戲服務器。請先使用 'connect' 命令。");
        }
        
        Ok(())
    }
    
    /// 狀態命令
    async fn cmd_status(&mut self) -> Result<()> {
        if let Some(client) = &self.game_client {
            let state = client.get_state();
            let game_state = client.get_game_state();
            
            println!("=== 遊戲狀態 ===");
            println!("客戶端狀態: {:?}", state);
            println!("{}", game_state.get_status_summary());
            
            // 顯示可用技能
            let available_abilities = game_state.get_available_abilities();
            println!("可用技能: {}", 
                     available_abilities.iter()
                         .map(|a| a.ability_id.as_str())
                         .collect::<Vec<_>>()
                         .join(", "));
        } else {
            println!("未連接到遊戲服務器");
        }
        
        Ok(())
    }
    
    /// 自動遊戲命令
    async fn cmd_auto(&mut self, duration: u64) -> Result<()> {
        if let Some(client) = &mut self.game_client {
            info!("開始自動遊戲模式，持續 {} 秒", duration);
            client.auto_play(duration).await?;
        } else {
            error!("未連接到遊戲服務器。請先使用 'connect' 命令。");
        }
        
        Ok(())
    }
    
    /// 演示命令
    async fn cmd_demo(&mut self) -> Result<()> {
        if let Some(client) = &mut self.game_client {
            info!("執行演示序列...");
            
            // 獲取演示序列
            let demo_sequence = client.get_game_state().local_player.name.clone();
            let player_simulator = crate::player::PlayerSimulator::new(
                demo_sequence, 
                client.get_game_state().local_player.hero_type.clone()
            );
            
            let sequence = player_simulator.get_demo_sequence();
            
            for (action, params) in sequence {
                info!("演示操作: {} - {}", action, params);
                if let Err(e) = client.perform_action(&action, params).await {
                    error!("演示操作失敗: {}", e);
                }
                
                // 等待一秒
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            
            info!("演示序列完成");
        } else {
            error!("未連接到遊戲服務器。請先使用 'connect' 命令。");
        }
        
        Ok(())
    }
    
    /// 技能列表命令
    async fn cmd_abilities(&mut self) -> Result<()> {
        println!("=== 可用英雄和技能 ===");
        
        println!("\n雜賀孫一 (saika_magoichi):");
        println!("  - sniper_mode: 狙擊模式");
        println!("  - saika_reinforcements: 雜賀眾");
        println!("  - rain_iron_cannon: 雨鐵炮");
        println!("  - three_stage_technique: 三段擊");
        
        println!("\n伊達政宗 (date_masamune):");
        println!("  - flame_blade: 火焰刀");
        println!("  - fire_dash: 火焰衝刺");
        println!("  - flame_assault: 火焰突擊");
        println!("  - matchlock_gun: 火繩槍");
        
        println!("\n使用方法:");
        println!("  omobaf cast <ability_id> --x <x> --y <y> --level <level>");
        
        Ok(())
    }
    
    /// 終端視圖命令
    async fn cmd_view(&mut self, radius: Option<f32>, width: Option<f32>, height: Option<f32>, show_vision: bool, live: bool) -> Result<()> {
        // 檢查遊戲客戶端狀態
        if self.game_client.is_none() {
            info!("❌ 未連接到遊戲服務器，正在嘗試自動連接...");
            
            // 載入配置
            let config = crate::config::AppConfig::load();
            
            // 如果設定自動啟動後端，先啟動後端
            if config.frontend.auto_start_backend {
                info!("自動啟動後端服務器...");
                let backend_manager = crate::backend_manager::BackendManager::new(config.clone());
                if let Err(e) = backend_manager.start().await {
                    error!("啟動後端失敗: {}", e);
                    return Err(e);
                }
                // 保存 backend_manager 引用以便稍後清理
                self.backend_manager = Some(backend_manager);
            }
            
            // 創建遊戲客戶端配置
            let client_config = crate::game_client::GameClientConfig {
                server_ip: config.server.mqtt_host,
                server_port: config.server.mqtt_port,
                client_id: "omobaf_viewer".to_string(),
                player_name: config.frontend.player_name,
                hero_type: config.frontend.hero_type,
            };
            
            // 自動連接和進入遊戲
            info!("🔗 開始自動連接到遊戲服務器...");
            self.cmd_connect(client_config.clone()).await?;
            
            info!("🎮 開始自動進入遊戲...");
            self.cmd_play(client_config).await?;
        } else {
            info!("✅ 遊戲客戶端已存在，跳過自動連接");
            if let Some(client) = &self.game_client {
                info!("🔍 當前遊戲客戶端狀態: {:?}", client.get_state());
            }
        }
        
        if let Some(client) = &mut self.game_client {
            // 創建終端視圖
            let view_result = if let Some(r) = radius {
                crate::terminal_view::TerminalView::new(r, show_vision)
            } else if let (Some(w), Some(h)) = (width, height) {
                crate::terminal_view::TerminalView::new_rect(w, h, show_vision)
            } else {
                // 默認值：半徑 20 的正方形視圖
                crate::terminal_view::TerminalView::new(20.0, show_vision)
            };
            
            match view_result {
                Ok(mut view) => {
                    if live {
                        info!("啟動實時終端視圖 (按 'q' 或 Esc 退出)");
                        if let Err(e) = view.init_terminal() {
                            error!("初始化終端失敗: {}", e);
                        } else {
                            loop {
                                // 同步共享遊戲狀態
                                if let Err(e) = client.sync_shared_state().await {
                                    error!("同步遊戲狀態失敗: {}", e);
                                }
                                
                                // 更新技能冷卻時間
                                client.get_game_state_mut().update_cooldowns(0.016); // 600ms = 0.6s
                                tokio::time::sleep(std::time::Duration::from_millis(16)).await;

                                match view.render_live(client.get_game_state()) {
                                    Ok(UserInput::Continue) => {
                                    }
                                    Ok(UserInput::Quit) => break, // 用戶按了退出鍵
                                    Ok(UserInput::Move(world_pos)) => {
                                        info!("移動到: ({:.1}, {:.1})", world_pos.x, world_pos.y);
                                        if let Err(e) = client.perform_action("move", serde_json::json!({
                                            "x": world_pos.x,
                                            "y": world_pos.y
                                        })).await {
                                            error!("移動指令失敗: {}", e);
                                        }
                                    }
                                    Ok(UserInput::Attack(world_pos)) => {
                                        info!("攻擊位置: ({:.1}, {:.1})", world_pos.x, world_pos.y);
                                        if let Err(e) = client.perform_action("attack", serde_json::json!({
                                            "target_position": [world_pos.x, world_pos.y],
                                            "attack_type": "basic"
                                        })).await {
                                            error!("攻擊指令失敗: {}", e);
                                        }
                                    }
                                    Ok(UserInput::MoveAttack(world_pos)) => {
                                        info!("移動攻擊到: ({:.1}, {:.1})", world_pos.x, world_pos.y);
                                        // 先移動再攻擊
                                        if let Err(e) = client.perform_action("move", serde_json::json!({
                                            "x": world_pos.x,
                                            "y": world_pos.y
                                        })).await {
                                            error!("移動攻擊移動部分失敗: {}", e);
                                        } else {
                                            // 短暫延遲後攻擊
                                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                                            if let Err(e) = client.perform_action("attack", serde_json::json!({
                                                "target_position": [world_pos.x, world_pos.y],
                                                "attack_type": "move_attack"
                                            })).await {
                                                error!("移動攻擊攻擊部分失敗: {}", e);
                                            }
                                        }
                                    }
                                    Ok(UserInput::ForceAttack(world_pos)) => {
                                        info!("強制攻擊位置: ({:.1}, {:.1})", world_pos.x, world_pos.y);
                                        if let Err(e) = client.perform_action("attack", serde_json::json!({
                                            "target_position": [world_pos.x, world_pos.y],
                                            "attack_type": "force_attack"
                                        })).await {
                                            error!("強制攻擊指令失敗: {}", e);
                                        }
                                    }
                                    Ok(UserInput::CastAbility(ability_id, world_pos)) => {
                                        info!("施放技能 {} 於位置: ({:.1}, {:.1})", ability_id, world_pos.x, world_pos.y);
                                        if let Err(e) = client.perform_action("cast_ability", serde_json::json!({
                                            "ability_id": ability_id,
                                            "target_position": [world_pos.x, world_pos.y],
                                            "level": 1
                                        })).await {
                                            error!("技能施放指令失敗: {}", e);
                                        }
                                    }
                                    Ok(UserInput::Cancel) => {
                                        // 技能選擇被取消，繼續遊戲循環
                                    }
                                    Ok(UserInput::UseItem(item_id, _target_pos)) => {
                                        info!("使用道具: {}", item_id);
                                        if let Err(e) = client.perform_action("use_item", serde_json::json!({
                                            "item_id": item_id
                                        })).await {
                                            error!("道具使用指令失敗: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        error!("終端視圖錯誤: {}", e);
                                        break;
                                    }
                                }
                            }
                            let _ = view.cleanup_terminal();
                        }
                    } else {
                        // 單次渲染
                        if let Err(e) = view.render(client.get_game_state()) {
                            error!("終端渲染錯誤: {}", e);
                        } else {
                            info!("按任意鍵退出...");
                            let _ = view.wait_for_key();
                            let _ = view.cleanup_terminal();
                        }
                    }
                }
                Err(e) => {
                    error!("創建終端視圖失敗: {}", e);
                }
            }
        } else {
            error!("未連接到遊戲服務器。請先使用 'connect' 命令。");
        }
        
        Ok(())
    }
    
    /// 斷開連接命令
    async fn cmd_disconnect(&mut self) -> Result<()> {
        if let Some(client) = &mut self.game_client {
            client.disconnect().await?;
            info!("已斷開遊戲服務器連接");
        }
        
        // 停止後端管理器
        if let Some(backend_manager) = &self.backend_manager {
            if let Err(e) = backend_manager.stop().await {
                warn!("停止後端時發生錯誤: {}", e);
            } else {
                info!("✅ 後端服務器已停止");
            }
        }
        
        self.game_client = None;
        self.backend_manager = None;
        Ok(())
    }
}