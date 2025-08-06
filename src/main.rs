/// omobaf - Open MOBA Frontend
/// 
/// 假遊戲前端客戶端，用於測試 omobab 後端的遊戲邏輯
use clap::Parser;
use log::error;

mod game_client;
mod mqtt_handler;
mod game_state;
mod player;
mod cli;
mod interactive;
mod terminal_view;
mod config;
mod backend_manager;
mod terminal_logger;

use cli::{Cli, CliHandler};
use interactive::InteractiveCli;

#[tokio::main]
async fn main() {
    // 解析命令行參數
    let args: Vec<String> = std::env::args().collect();
    
    // 如果沒有參數，啟動互動式模式
    if args.len() == 1 {
        // 初始化日誌
        env_logger::init();
        
        // 啟動互動式 CLI
        let mut interactive = InteractiveCli::new();
        if let Err(e) = interactive.run().await {
            error!("互動式 CLI 錯誤: {}", e);
            std::process::exit(1);
        }
    } else {
        // 使用原本的命令行模式
        let cli = Cli::parse();
        
        // 創建 CLI 處理器
        let mut handler = CliHandler::new();
        
        // 處理命令
        match handler.handle_command(cli).await {
            Ok(_) => {},
            Err(e) => {
                error!("命令執行失敗: {}", e);
                std::process::exit(1);
            }
        }
    }
}
