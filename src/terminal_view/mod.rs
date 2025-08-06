/// 終端視圖模塊
/// 
/// 使用 crossterm 提供跨平台終端控制和豐富的視覺效果

pub mod display;
pub mod input;
pub mod renderer;
pub mod viewport;

use std::io;
use crossterm::terminal;
use vek::Vec2;
use crate::game_state::GameState;

pub use display::MapDisplay;
pub use input::{UserInput, InputHandler};
pub use renderer::MapRenderer;
pub use viewport::ViewportManager;

/// 終端視圖主控制器
pub struct TerminalView {
    /// 視口管理器
    pub viewport: ViewportManager,
    /// 地圖渲染器
    pub renderer: MapRenderer,
    /// 輸入處理器
    pub input_handler: InputHandler,
    /// 是否顯示視野範圍
    pub show_vision: bool,
    /// 終端寬度（字符數）
    pub terminal_width: u16,
    /// 終端高度（字符數）
    pub terminal_height: u16,
}

impl TerminalView {
    /// 創建新的終端視圖（從半徑創建正方形視圖）
    pub fn new(radius: f32, show_vision: bool) -> io::Result<Self> {
        let view_size = radius * 2.0;
        let (width, height) = terminal::size()?;
        
        Ok(Self {
            viewport: ViewportManager::new(view_size, view_size),
            renderer: MapRenderer::new(),
            input_handler: InputHandler::new(),
            show_vision,
            terminal_width: width,
            terminal_height: height.saturating_sub(3), // 留出日誌區域空間
        })
    }
    
    /// 創建指定寬高的終端視圖
    pub fn new_rect(width: f32, height: f32, show_vision: bool) -> io::Result<Self> {
        let (term_width, term_height) = terminal::size()?;
        
        Ok(Self {
            viewport: ViewportManager::new(width, height),
            renderer: MapRenderer::new(),
            input_handler: InputHandler::new(),
            show_vision,
            terminal_width: term_width,
            terminal_height: term_height.saturating_sub(3),
        })
    }
    
    /// 初始化終端
    pub fn init_terminal(&self) -> io::Result<()> {
        self.renderer.init_terminal()
    }
    
    /// 清理終端
    pub fn cleanup_terminal(&self) -> io::Result<()> {
        self.renderer.cleanup_terminal()
    }
    
    /// 渲染終端視圖
    pub fn render(&self, game_state: &GameState) -> io::Result<()> {
        self.renderer.render(
            game_state,
            &self.viewport,
            self.show_vision,
            self.terminal_width,
            self.terminal_height
        )
    }
    
    /// 等待用戶按鍵
    pub fn wait_for_key(&self) -> io::Result<crossterm::event::KeyEvent> {
        self.input_handler.wait_for_key()
    }
    
    /// 實時模式循環
    pub fn render_live(&mut self, game_state: &GameState) -> io::Result<UserInput> {
        // 渲染當前狀態
        self.render(game_state)?;
        
        // 處理用戶輸入
        self.input_handler.handle_input(
            game_state,
            &self.viewport,
            self.terminal_width,
            self.terminal_height
        )
    }
}