/// 互動式會話管理
use std::io::{self, Write};
use anyhow::Result;
use log::warn;
use colored::*;

use crate::game_client::{GameClientConfig, ClientState};
use crate::terminal_view::{TerminalView, UserInput};
use crate::config::AppConfig;
use super::commands::CommandHandler;

/// 互動式 CLI 處理器
pub struct InteractiveCli {
    command_handler: CommandHandler,
    running: bool,
}

impl InteractiveCli {
    /// 創建新的互動式 CLI
    pub fn new() -> Self {
        let app_config = AppConfig::load();
        let config = GameClientConfig::default();
        
        Self {
            command_handler: CommandHandler::new(config, app_config),
            running: true,
        }
    }
    
    /// 啟動互動式 CLI
    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();
        
        // 自動啟動後端（如果配置了的話）
        if let Some(ref backend_manager) = self.command_handler.backend_manager {
            println!("🚀 自動啟動後端...");
            match backend_manager.start().await {
                Ok(_) => {
                    println!("✅ 後端已啟動");
                },
                Err(e) => {
                    println!("⚠️  無法啟動後端: {}。將嘗試連接現有後端。", e);
                }
            }
        }
        
        // 自動嘗試連接到本地端
        println!("🔗 自動連接到本地端...");
        match self.command_handler.auto_connect_localhost().await {
            Ok(_) => {
                println!("✅ 已連接到 127.0.0.1:1883");
            },
            Err(e) => {
                println!("⚠️  無法連接到本地端: {}。請手動使用 'connect' 命令。", e);
            }
        }
        println!();
        
        while self.running {
            self.print_prompt();
            
            let input = self.read_input()?;
            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            
            if parts.is_empty() {
                continue;
            }
            
            match self.handle_command(&parts).await {
                Ok(_) => {},
                Err(e) => {
                    println!("{} {}", "錯誤:".red(), e);
                }
            }
        }
        
        Ok(())
    }
    
    /// 打印歡迎訊息
    fn print_welcome(&self) {
        println!("\n{}", "=".repeat(60).bright_blue());
        println!("{}", "      Open MOBA Frontend - 互動式客戶端".bright_cyan().bold());
        println!("{}", "=".repeat(60).bright_blue());
        println!("\n輸入 {} 查看可用命令\n", "help".yellow());
    }
    
    /// 打印提示符
    fn print_prompt(&self) {
        let status = match &self.command_handler.game_client {
            Some(client) => match client.get_state() {
                ClientState::Connected => "[已連接]".green(),
                ClientState::InGame => "[遊戲中]".bright_green(),
                ClientState::Connecting => "[連接中]".yellow(),
                ClientState::Disconnected => "[未連接]".red(),
                ClientState::Error(_) => "[錯誤]".bright_red(),
            },
            None => "[未連接]".red(),
        };
        
        print!("{} {} ", status, ">".bright_white());
        io::stdout().flush().unwrap();
    }
    
    /// 讀取用戶輸入
    fn read_input(&self) -> Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input)
    }
    
    /// 處理命令
    async fn handle_command(&mut self, parts: &[&str]) -> Result<()> {
        let command = parts[0];
        
        match command {
            "help" | "?" => self.show_help(),
            "connect" => self.command_handler.handle_connect(parts).await?,
            "disconnect" => self.command_handler.handle_disconnect().await?,
            "config" => self.command_handler.handle_config(parts)?,
            "status" => self.command_handler.handle_status()?,
            "play" => self.command_handler.handle_play(parts).await?,
            "move" => self.command_handler.handle_move(parts).await?,
            "cast" => self.command_handler.handle_cast(parts).await?,
            "attack" => self.command_handler.handle_attack(parts).await?,
            "abilities" => self.command_handler.handle_abilities()?,
            "auto" => self.command_handler.handle_auto(parts).await?,
            "view" => self.handle_view(parts).await?,
            "viewport" => self.command_handler.handle_viewport(parts).await?,
            "zoom" => self.command_handler.handle_zoom(parts).await?,
            "backend" => self.command_handler.handle_backend(parts).await?,
            "clear" => self.clear_screen(),
            "exit" | "quit" => self.handle_exit().await?,
            _ => {
                println!("{} 未知命令: {}。輸入 {} 查看幫助。", 
                    "!".red(), command, "help".yellow());
            }
        }
        
        Ok(())
    }
    
    /// 顯示幫助
    fn show_help(&self) {
        println!("\n{}", "可用命令:".bright_cyan().bold());
        println!("{}", "-".repeat(40).bright_black());
        
        println!("  {} - 顯示此幫助訊息", "help, ?".green());
        println!("  {} <ip> [port] - 連接到服務器", "connect".green());
        println!("  {} - 斷開連接", "disconnect".green());
        println!("  {} [key] [value] - 查看或修改配置", "config".green());
        println!("  {} - 查看當前狀態", "status".green());
        println!("  {} [hero] - 開始遊戲", "play".green());
        println!("  {} <x> <y> - 移動到指定位置", "move".green());
        println!("  {} <ability> [x] [y] [level] - 施放技能", "cast".green());
        println!("  {} <x> <y> - 攻擊指定位置", "attack".green());
        println!("  {} - 列出可用技能", "abilities".green());
        println!("  {} [duration] - 自動遊戲模式", "auto".green());
        println!("  {} [size] [--vision] [--live] - 顯示終端地圖視圖 (支援滑鼠操作)", "view".green());
        println!("  {} [width] [height] - 設置視窗大小", "viewport".green());
        println!("  {} <level> - 設置縮放等級 (0.5-3.0)", "zoom".green());
        println!("  {} <start|stop|restart|status> - 後端管理", "backend".green());
        println!("  {} - 清除畫面", "clear".green());
        println!("  {} - 退出程式", "exit, quit".green());
        
        println!("\n{}", "滑鼠控制 (在實時視圖中):".bright_cyan().bold());
        println!("  左鍵點擊 - 移動到目標位置");
        println!("  右鍵點擊 - 攻擊目標位置");
        println!("  Shift+左鍵 - 移動攻擊");
        println!("  Ctrl+左鍵 - 強制攻擊");
        
        println!("\n{}", "鍵盤技能控制 (在實時視圖中):".bright_cyan().bold());
        println!("  {} - 選擇技能後左鍵點擊施放", "W/E/R/T".yellow());
        println!("  {} - 根據當前英雄自動對應技能", "W/E/R/T".green());
        println!("  {} - 雜賀孫市: W=狙擊模式 E=雜賀眾 R=雨鐵炮 T=三段擊", "技能對應".cyan());
        println!("  {} - 伊達政宗: W=火焰刀 E=火焰衝刺 R=火焰突擊 T=火繩槍", "技能對應".cyan());
        
        println!("\n{}", "道具控制 (在實時視圖中):".bright_cyan().bold());
        println!("  {} - 直接使用對應道具", "1-9".yellow());
        println!("  {} - 1=生命藥水 2=魔力藥水 3=傳送卷軸 4=煙霧彈", "道具對應".cyan());
        println!("  {} - 狀態欄顯示: [1]生命 (5) 表示1號位生命藥水剩餘5個", "顯示說明".magenta());
        
        println!("\n{}", "範例:".bright_cyan().bold());
        println!("  connect localhost 1883");
        println!("  play saika_magoichi");
        println!("  move 100 200");
        println!("  cast sniper_mode 150 250 1");
        println!("  view 25 --vision");
        println!("  view 30 --live  # 支援滑鼠操作");
        println!();
    }
    
    /// 處理終端視圖命令
    async fn handle_view(&mut self, parts: &[&str]) -> Result<()> {
        // 檢查是否有客戶端連接
        if self.command_handler.game_client.is_none() {
            return Err(anyhow::anyhow!("請先連接到服務器"));
        }
        
        // 解析參數
        let mut size = 20.0;  // 默認正方形大小
        let mut width: Option<f32> = None;
        let mut height: Option<f32> = None;
        let mut show_vision = false;
        let mut live_mode = false;
        
        // 解析命令行參數
        let mut i = 1;
        while i < parts.len() {
            match parts[i] {
                "--vision" => show_vision = true,
                "--live" => live_mode = true,
                arg if arg.parse::<f32>().is_ok() => {
                    let val = arg.parse::<f32>()?;
                    if width.is_none() {
                        // 第一個數字：可能是大小或寬度
                        if i + 1 < parts.len() && parts[i + 1].parse::<f32>().is_ok() {
                            // 下一個也是數字，這個是寬度
                            width = Some(val);
                        } else {
                            // 下一個不是數字，這個是正方形大小
                            size = val;
                        }
                    } else if height.is_none() {
                        // 第二個數字：高度
                        height = Some(val);
                    }
                },
                _ => {
                    println!("{} 未知參數: {}", "!".yellow(), parts[i]);
                }
            }
            i += 1;
        }
        
        // 創建終端視圖
        let view_result = if let (Some(w), Some(h)) = (width, height) {
            TerminalView::new_rect(w, h, show_vision)
        } else {
            TerminalView::new(size, show_vision)
        };
        
        match view_result {
            Ok(mut view) => {
                if live_mode {
                    self.run_live_view(&mut view, size, width, height, show_vision).await?;
                } else {
                    self.run_static_view(&mut view, size, width, height).await?;
                }
            }
            Err(e) => {
                println!("{} 創建終端視圖失敗: {}", "❌".red(), e);
            }
        }
        
        Ok(())
    }
    
    /// 運行實時視圖模式
    async fn run_live_view(
        &mut self,
        view: &mut TerminalView,
        size: f32,
        width: Option<f32>,
        height: Option<f32>,
        show_vision: bool,
    ) -> Result<()> {
        let view_desc = if let (Some(w), Some(h)) = (width, height) {
            format!("{}x{}", w, h)
        } else {
            format!("{:.0}x{:.0}", size * 2.0, size * 2.0)
        };
        
        println!("{} 啟動實時終端視圖 (按 {} 退出)", 
                 "🖥️".bright_white(), "'q' 或 Esc".yellow());
        println!("視圖範圍: {}, 顯示視野: {}\n", view_desc, if show_vision { "是" } else { "否" });
        
        if let Err(e) = view.init_terminal() {
            println!("{} 初始化終端失敗: {}", "❌".red(), e);
            return Ok(());
        }
        
        // 實時循環
        loop {
            // 同步共享遊戲狀態
            if let Some(client) = self.command_handler.game_client.as_mut() {
                if let Err(e) = client.sync_shared_state().await {
                    println!("{} 同步遊戲狀態失敗: {}", "❌".red(), e);
                }
                
                // 更新技能冷卻時間
                client.get_game_state_mut().update_cooldowns(0.6); // 600ms = 0.6s
            }
            
            // 渲染視圖
            let render_result = if let Some(client) = self.command_handler.game_client.as_ref() {
                view.render_live(client.get_game_state())
            } else {
                break; // 沒有客戶端連接，退出循環
            };
            
            match render_result {
                Ok(UserInput::Continue) => {
                    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                }
                Ok(UserInput::Quit) => break, // 用戶按了退出鍵
                Ok(input) => {
                    // 處理用戶輸入動作
                    if let Err(e) = self.command_handler.handle_view_input(input).await {
                        println!("{} 處理輸入失敗: {}", "❌".red(), e);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                }
                Err(e) => {
                    println!("{} 終端視圖錯誤: {}", "❌".red(), e);
                    break;
                }
            }
        }
        let _ = view.cleanup_terminal();
        println!("{} 退出實時視圖模式", "✓".green());
        
        Ok(())
    }
    
    /// 運行靜態視圖模式
    async fn run_static_view(
        &mut self,
        view: &mut TerminalView,
        size: f32,
        width: Option<f32>,
        height: Option<f32>,
    ) -> Result<()> {
        let view_desc = if let (Some(w), Some(h)) = (width, height) {
            format!("{}x{}", w, h)
        } else {
            format!("{:.0}x{:.0}", size * 2.0, size * 2.0)
        };
        
        println!("{} 終端地圖視圖 (範圍: {})", 
                 "🗺️".bright_white(), view_desc);
        
        let client = self.command_handler.game_client.as_ref().unwrap();
        
        if let Err(e) = view.render(client.get_game_state()) {
            println!("{} 終端渲染錯誤: {}", "❌".red(), e);
        } else {
            println!("\n{} 按任意鍵繼續...", "💡".bright_white());
            let _ = view.wait_for_key();
            let _ = view.cleanup_terminal();
        }
        
        Ok(())
    }
    
    /// 清除畫面
    fn clear_screen(&self) {
        print!("\x1B[2J\x1B[1;1H");
        self.print_welcome();
    }
    
    /// 處理退出命令
    async fn handle_exit(&mut self) -> Result<()> {
        // 停止後端程序（如果由我們管理的話）
        if let Some(ref backend_manager) = self.command_handler.backend_manager {
            if backend_manager.is_running().await {
                println!("{} 停止後端程序...", "→".yellow());
                if let Err(e) = backend_manager.stop().await {
                    warn!("停止後端程序時發生錯誤: {}", e);
                }
            }
        }
        
        if let Some(mut client) = self.command_handler.game_client.take() {
            println!("{} 斷開連接...", "→".yellow());
            client.disconnect().await?;
        }
        
        println!("{} 再見！", "👋".bright_white());
        self.running = false;
        Ok(())
    }
}