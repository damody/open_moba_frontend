/// 命令處理模塊
use std::io::{self, Write};
use anyhow::Result;
use colored::*;
use crate::game_client::{GameClient, GameClientConfig, ClientState};
use crate::config::AppConfig;
use crate::backend_manager::BackendManager;
use crate::terminal_view::UserInput;

/// 命令處理器
pub struct CommandHandler {
    pub game_client: Option<GameClient>,
    pub config: GameClientConfig,
    pub app_config: AppConfig,
    pub backend_manager: Option<BackendManager>,
}

impl CommandHandler {
    /// 創建新的命令處理器
    pub fn new(config: GameClientConfig, app_config: AppConfig) -> Self {
        Self {
            game_client: None,
            config,
            backend_manager: if app_config.frontend.auto_start_backend {
                Some(BackendManager::new(app_config.clone()))
            } else {
                None
            },
            app_config,
        }
    }
    
    /// 自動連接到本地端
    pub async fn auto_connect_localhost(&mut self) -> Result<()> {
        let mut client = GameClient::new(self.config.clone());
        client.connect().await?;
        self.game_client = Some(client);
        Ok(())
    }
    
    /// 處理連接命令
    pub async fn handle_connect(&mut self, parts: &[&str]) -> Result<()> {
        let ip = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            self.config.server_ip.clone()
        };
        
        let port = if parts.len() > 2 {
            parts[2].parse()?
        } else {
            self.config.server_port
        };
        
        println!("{} 連接到 {}:{}...", "→".green(), ip, port);
        
        self.config.server_ip = ip;
        self.config.server_port = port;
        
        let mut client = GameClient::new(self.config.clone());
        client.connect().await?;
        
        self.game_client = Some(client);
        
        println!("{} 連接成功！", "✓".green());
        Ok(())
    }
    
    /// 處理斷開連接命令
    pub async fn handle_disconnect(&mut self) -> Result<()> {
        if let Some(mut client) = self.game_client.take() {
            client.disconnect().await?;
            println!("{} 已斷開連接", "✓".green());
        } else {
            println!("{} 尚未連接到服務器", "!".yellow());
        }
        Ok(())
    }
    
    /// 處理配置命令
    pub fn handle_config(&mut self, parts: &[&str]) -> Result<()> {
        if parts.len() == 1 {
            // 顯示當前配置
            println!("\n{}", "當前配置:".bright_cyan().bold());
            println!("  服務器: {}:{}", self.config.server_ip, self.config.server_port);
            println!("  客戶端ID: {}", self.config.client_id);
            println!("  玩家名稱: {}", self.config.player_name);
            println!("  英雄類型: {}", self.config.hero_type);
        } else if parts.len() >= 3 {
            // 修改配置
            let key = parts[1];
            let value = parts[2..].join(" ");
            
            match key {
                "server" | "ip" => {
                    self.config.server_ip = value;
                    println!("{} 服務器 IP 設為: {}", "✓".green(), self.config.server_ip);
                },
                "port" => {
                    self.config.server_port = value.parse()?;
                    println!("{} 服務器端口設為: {}", "✓".green(), self.config.server_port);
                },
                "name" | "player" => {
                    self.config.player_name = value;
                    println!("{} 玩家名稱設為: {}", "✓".green(), self.config.player_name);
                },
                "hero" => {
                    self.config.hero_type = value;
                    println!("{} 英雄類型設為: {}", "✓".green(), self.config.hero_type);
                },
                _ => {
                    println!("{} 未知配置項: {}", "!".red(), key);
                }
            }
        }
        Ok(())
    }
    
    /// 處理狀態命令
    pub fn handle_status(&self) -> Result<()> {
        println!("\n{}", "遊戲狀態:".bright_cyan().bold());
        println!("{}", "-".repeat(40).bright_black());
        
        match &self.game_client {
            Some(client) => {
                let state = client.get_state();
                println!("  連接狀態: {}", format!("{:?}", state).bright_white());
                
                if let ClientState::InGame = state {
                    let game_state = client.get_game_state();
                    println!("  玩家: {}", game_state.local_player.name.bright_yellow());
                    println!("  英雄: {}", game_state.local_player.hero_type.bright_yellow());
                    println!("  位置: ({:.1}, {:.1})", 
                        game_state.local_player.position.x,
                        game_state.local_player.position.y);
                    println!("  生命值: {:.0}/{:.0}", 
                        game_state.local_player.health.0, 
                        game_state.local_player.health.1);
                }
            },
            None => {
                println!("  狀態: {}", "未連接".red());
            }
        }
        
        Ok(())
    }
    
    /// 處理開始遊戲命令
    pub async fn handle_play(&mut self, parts: &[&str]) -> Result<()> {
        if let Some(client) = &mut self.game_client {
            if parts.len() > 1 {
                self.config.hero_type = parts[1].to_string();
            }
            
            println!("{} 開始遊戲，英雄: {}", "→".green(), self.config.hero_type);
            client.enter_game().await?;
            println!("{} 已進入遊戲！", "✓".green());
        } else {
            return Err(anyhow::anyhow!("請先連接到服務器"));
        }
        Ok(())
    }
    
    /// 處理移動命令
    pub async fn handle_move(&mut self, parts: &[&str]) -> Result<()> {
        if parts.len() < 3 {
            return Err(anyhow::anyhow!("用法: move <x> <y>"));
        }
        
        let x: f32 = parts[1].parse()?;
        let y: f32 = parts[2].parse()?;
        
        if let Some(client) = &mut self.game_client {
            println!("{} 移動到 ({}, {})", "→".green(), x, y);
            client.perform_action("move", serde_json::json!({
                "x": x,
                "y": y
            })).await?;
            println!("{} 移動完成", "✓".green());
        } else {
            return Err(anyhow::anyhow!("請先連接到服務器"));
        }
        
        Ok(())
    }
    
    /// 處理施放技能命令
    pub async fn handle_cast(&mut self, parts: &[&str]) -> Result<()> {
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("用法: cast <ability> [x] [y] [level]"));
        }
        
        let ability_id = parts[1];
        let x = if parts.len() > 2 { Some(parts[2].parse::<f32>()?) } else { None };
        let y = if parts.len() > 3 { Some(parts[3].parse::<f32>()?) } else { None };
        let level = if parts.len() > 4 { parts[4].parse::<u8>()? } else { 1 };
        
        if let Some(client) = &mut self.game_client {
            println!("{} 施放技能: {}", "→".green(), ability_id);
            
            let mut params = serde_json::json!({
                "ability_id": ability_id,
                "level": level
            });
            
            if let (Some(x), Some(y)) = (x, y) {
                params["target_position"] = serde_json::json!([x, y]);
            }
            
            client.perform_action("cast_ability", params).await?;
            println!("{} 技能施放成功", "✓".green());
        } else {
            return Err(anyhow::anyhow!("請先連接到服務器"));
        }
        
        Ok(())
    }
    
    /// 處理攻擊命令
    pub async fn handle_attack(&mut self, parts: &[&str]) -> Result<()> {
        if parts.len() < 3 {
            return Err(anyhow::anyhow!("用法: attack <x> <y>"));
        }
        
        let x: f32 = parts[1].parse()?;
        let y: f32 = parts[2].parse()?;
        
        if let Some(client) = &mut self.game_client {
            println!("{} 攻擊位置 ({}, {})", "→".green(), x, y);
            client.perform_action("attack", serde_json::json!({
                "target_position": [x, y]
            })).await?;
            println!("{} 攻擊完成", "✓".green());
        } else {
            return Err(anyhow::anyhow!("請先連接到服務器"));
        }
        
        Ok(())
    }
    
    /// 處理自動遊戲命令
    pub async fn handle_auto(&mut self, parts: &[&str]) -> Result<()> {
        let duration = if parts.len() > 1 {
            parts[1].parse()?
        } else {
            30 // 默認 30 秒
        };
        
        if let Some(client) = &mut self.game_client {
            println!("{} 開始自動遊戲模式，持續 {} 秒", "→".green(), duration);
            client.auto_play(duration).await?;
            println!("{} 自動遊戲結束", "✓".green());
        } else {
            return Err(anyhow::anyhow!("請先連接到服務器"));
        }
        
        Ok(())
    }
    
    /// 處理視窗設置命令
    pub async fn handle_viewport(&mut self, parts: &[&str]) -> Result<()> {
        if let Some(client) = &mut self.game_client {
            if parts.len() == 1 {
                // 顯示當前視窗設置
                let viewport = &client.get_game_state().viewport;
                println!("\n{}", "當前視窗設置:".bright_cyan().bold());
                println!("  中心: ({:.1}, {:.1})", viewport.center.x, viewport.center.y);
                println!("  尺寸: {:.0} x {:.0}", viewport.width, viewport.height);
                println!("  縮放: {:.1}x", viewport.zoom);
                
                let (min, max) = viewport.get_bounds();
                println!("  範圍: ({:.1}, {:.1}) 到 ({:.1}, {:.1})", 
                    min.x, min.y, max.x, max.y);
            } else if parts.len() >= 3 {
                // 設置新的視窗大小
                let width: f32 = parts[1].parse()?;
                let height: f32 = parts[2].parse()?;
                
                client.get_game_state_mut().viewport.set_size(width, height);
                client.send_viewport_update().await?;
                
                println!("{} 視窗大小設為: {:.0} x {:.0}", "✓".green(), width, height);
            } else {
                return Err(anyhow::anyhow!("用法: viewport [width] [height]"));
            }
        } else {
            return Err(anyhow::anyhow!("請先連接到服務器"));
        }
        
        Ok(())
    }
    
    /// 處理縮放命令
    pub async fn handle_zoom(&mut self, parts: &[&str]) -> Result<()> {
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("用法: zoom <level>"));
        }
        
        let zoom: f32 = parts[1].parse()?;
        
        if let Some(client) = &mut self.game_client {
            client.get_game_state_mut().viewport.set_zoom(zoom);
            client.send_viewport_update().await?;
            
            println!("{} 縮放設為: {:.1}x", "✓".green(), zoom);
            
            let (min, max) = client.get_game_state().viewport.get_bounds();
            println!("  新視窗範圍: ({:.1}, {:.1}) 到 ({:.1}, {:.1})", 
                min.x, min.y, max.x, max.y);
        } else {
            return Err(anyhow::anyhow!("請先連接到服務器"));
        }
        
        Ok(())
    }
    
    /// 處理後端管理命令
    pub async fn handle_backend(&mut self, parts: &[&str]) -> Result<()> {
        if parts.len() < 2 {
            println!("\n{}", "後端管理命令:".bright_cyan().bold());
            println!("  {} - 啟動後端", "backend start".green());
            println!("  {} - 停止後端", "backend stop".green());
            println!("  {} - 重啟後端", "backend restart".green());
            println!("  {} - 查看後端狀態", "backend status".green());
            return Ok(());
        }

        let action = parts[1];

        if self.backend_manager.is_none() {
            println!("{} 後端自動管理未啟用。請在 config.toml 中設置 auto_start_backend = true", "⚠️".yellow());
            return Ok(());
        }

        let backend_manager = self.backend_manager.as_ref().unwrap();

        match action {
            "start" => {
                println!("{} 啟動後端...", "🚀".bright_white());
                match backend_manager.start().await {
                    Ok(_) => println!("{} 後端已啟動", "✅".green()),
                    Err(e) => println!("{} 啟動失敗: {}", "❌".red(), e),
                }
            },
            "stop" => {
                println!("{} 停止後端...", "🛑".bright_white());
                match backend_manager.stop().await {
                    Ok(_) => println!("{} 後端已停止", "✅".green()),
                    Err(e) => println!("{} 停止失敗: {}", "❌".red(), e),
                }
            },
            "restart" => {
                println!("{} 重啟後端...", "🔄".bright_white());
                match backend_manager.restart().await {
                    Ok(_) => println!("{} 後端已重啟", "✅".green()),
                    Err(e) => println!("{} 重啟失敗: {}", "❌".red(), e),
                }
            },
            "status" => {
                let is_running = backend_manager.is_running().await;
                let pid = backend_manager.get_pid().await;
                
                println!("\n{}", "後端狀態:".bright_cyan().bold());
                println!("  狀態: {}", if is_running {
                    "運行中".green()
                } else {
                    "已停止".red()
                });
                
                if let Some(pid) = pid {
                    println!("  進程 ID: {}", pid.to_string().yellow());
                }
                
                println!("  執行檔路徑: {}", self.app_config.backend.executable_path.cyan());
                
                if !self.app_config.backend.args.is_empty() {
                    println!("  啟動參數: {}", self.app_config.backend.args.join(" ").cyan());
                }
            },
            _ => {
                println!("{} 未知的後端命令: {}。使用 'backend' 查看可用命令。", "!".red(), action);
            }
        }

        Ok(())
    }
    
    /// 處理技能列表命令
    pub fn handle_abilities(&self) -> Result<()> {
        println!("\n{}", "可用英雄和技能:".bright_cyan().bold());
        println!("{}", "-".repeat(40).bright_black());
        
        println!("\n{} (saika_magoichi):", "雜賀孫市".bright_yellow());
        println!("  • {} - 狙擊模式", "sniper_mode".green());
        println!("  • {} - 雜賀眾", "saika_reinforcements".green());
        println!("  • {} - 雨鐵炮", "rain_iron_cannon".green());
        println!("  • {} - 三段擊", "three_stage_technique".green());
        
        println!("\n{} (date_masamune):", "伊達政宗".bright_yellow());
        println!("  • {} - 火焰刀", "flame_blade".green());
        println!("  • {} - 火焰衝刺", "fire_dash".green());
        println!("  • {} - 火焰突擊", "flame_assault".green());
        println!("  • {} - 火繩槍", "matchlock_gun".green());
        
        Ok(())
    }
    
    /// 處理實時視圖輸入動作
    pub async fn handle_view_input(&mut self, input: UserInput) -> Result<()> {
        if let Some(client) = &mut self.game_client {
            match input {
                UserInput::Move(world_pos) => {
                    println!("{} 移動到: ({:.1}, {:.1})", "🚶".bright_green(), world_pos.x, world_pos.y);
                    client.perform_action("move", serde_json::json!({
                        "x": world_pos.x,
                        "y": world_pos.y
                    })).await?;
                }
                UserInput::Attack(world_pos) => {
                    println!("{} 攻擊位置: ({:.1}, {:.1})", "⚔️".bright_red(), world_pos.x, world_pos.y);
                    client.perform_action("attack", serde_json::json!({
                        "target_position": [world_pos.x, world_pos.y],
                        "attack_type": "basic"
                    })).await?;
                }
                UserInput::MoveAttack(world_pos) => {
                    println!("{} 移動攻擊到: ({:.1}, {:.1})", "🏃⚔️".bright_yellow(), world_pos.x, world_pos.y);
                    // 先移動再攻擊
                    client.perform_action("move", serde_json::json!({
                        "x": world_pos.x,
                        "y": world_pos.y
                    })).await?;
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    client.perform_action("attack", serde_json::json!({
                        "target_position": [world_pos.x, world_pos.y],
                        "attack_type": "move_attack"
                    })).await?;
                }
                UserInput::ForceAttack(world_pos) => {
                    println!("{} 強制攻擊位置: ({:.1}, {:.1})", "💥".bright_red(), world_pos.x, world_pos.y);
                    client.perform_action("attack", serde_json::json!({
                        "target_position": [world_pos.x, world_pos.y],
                        "attack_type": "force_attack"
                    })).await?;
                }
                UserInput::CastAbility(ability_id, world_pos) => {
                    println!("{} 施放技能 {} 於位置: ({:.1}, {:.1})", "✨".bright_magenta(), ability_id, world_pos.x, world_pos.y);
                    client.perform_action("cast_ability", serde_json::json!({
                        "ability_id": ability_id,
                        "target_position": [world_pos.x, world_pos.y],
                        "level": 1
                    })).await?;
                }
                UserInput::UseItem(item_id, _target_pos) => {
                    println!("{} 使用道具: {}", "🧪".bright_blue(), item_id);
                    client.perform_action("use_item", serde_json::json!({
                        "item_id": item_id
                    })).await?;
                }
                _ => {} // Continue 和 Cancel 不需要處理
            }
        }
        Ok(())
    }
}