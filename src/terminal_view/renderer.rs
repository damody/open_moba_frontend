use super::{MapDisplay, ViewportManager};
use crate::game_state::{EntityType, GameState};
use crossterm::{
    cursor, event, execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
/// 地圖渲染模塊
use std::io::{self, Write};
use vek::Vec2;

/// 地圖渲染器
pub struct MapRenderer;

impl MapRenderer {
    /// 創建新的地圖渲染器
    pub fn new() -> Self {
        Self
    }

    /// 初始化終端
    pub fn init_terminal(&self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(
            io::stdout(),
            terminal::EnterAlternateScreen,
            Clear(ClearType::All),
            cursor::Hide,
            event::EnableMouseCapture
        )?;
        // 確保輸入緩衝區被清空
        while event::poll(std::time::Duration::from_millis(0))? {
            let _ = event::read()?;
        }
        Ok(())
    }

    /// 清理終端
    pub fn cleanup_terminal(&self) -> io::Result<()> {
        execute!(
            io::stdout(),
            event::DisableMouseCapture,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    /// 渲染終端視圖
    pub fn render(
        &self,
        game_state: &GameState,
        viewport: &ViewportManager,
        show_vision: bool,
        terminal_width: u16,
        terminal_height: u16,
    ) -> io::Result<()> {
        let mut stdout = io::stdout();

        // 只在初次渲染時清除螢幕，之後使用 cursor 移動
        queue!(stdout, cursor::MoveTo(0, 0))?;

        // 檢查是否有有效的遊戲資料
        if !game_state.has_valid_data() {
            // 沒有資料時顯示等待畫面
            self.render_waiting_screen(
                &mut stdout,
                game_state,
                viewport,
                show_vision,
                terminal_width,
                terminal_height,
            )?;
        } else {
            // 有資料時正常渲染
            log::info!("有資料時正常渲染...");
            // 創建地圖網格
            let mut map_grid =
                self.create_map_grid(game_state, viewport, terminal_width, terminal_height);

            // 渲染玩家和實體
            self.render_entities(
                game_state,
                &mut map_grid,
                viewport,
                terminal_width,
                terminal_height,
            );

            // 渲染視野範圍（如果啟用）
            if show_vision {
                self.render_vision_range(
                    &mut map_grid,
                    game_state,
                    terminal_width,
                    terminal_height,
                );
            }

            // 輸出地圖到終端
            self.print_map(&mut stdout, &map_grid)?;

            // 顯示底部日誌
            self.print_logs(&mut stdout, terminal_width, terminal_height)?;
        }

        stdout.flush()?;
        Ok(())
    }

    /// 渲染等待畫面
    fn render_waiting_screen(
        &self,
        stdout: &mut io::Stdout,
        game_state: &GameState,
        viewport: &ViewportManager,
        show_vision: bool,
        terminal_width: u16,
        terminal_height: u16,
    ) -> io::Result<()> {
        let width = terminal_width as usize;
        let height = terminal_height as usize;

        // 創建空白地圖網格
        let mut map_grid = vec![vec![MapDisplay::EMPTY; width]; height];
        // 在地圖中心顯示等待訊息
        self.render_waiting_message(&mut map_grid, width, height);

        // 輸出地圖到終端
        self.print_map(stdout, &map_grid)?;

        // 顯示底部日誌
        self.print_logs(stdout, terminal_width, terminal_height)?;

        Ok(())
    }

    /// 在地圖中心渲染等待訊息
    fn render_waiting_message(&self, grid: &mut Vec<Vec<MapDisplay>>, width: usize, height: usize) {
        let messages = vec![
            "在地圖中心渲染等待訊息...",
            "Please ensure connected to game server",
            "and entered game mode",
        ];

        let center_y = height / 2;
        let start_y = center_y.saturating_sub(messages.len() / 2);

        for (i, message) in messages.iter().enumerate() {
            let y = start_y + i;
            if y < height {
                let msg_len = message.len();
                let center_x = width / 2;
                let start_x = center_x.saturating_sub(msg_len / 2);

                // 清除該行的背景
                for x in 0..width {
                    if grid[y][x].symbol == MapDisplay::EMPTY.symbol {
                        grid[y][x] = MapDisplay {
                            symbol: ' ',
                            color: Color::DarkGrey,
                        };
                    }
                }

                // 渲染訊息文字
                for (j, ch) in message.chars().enumerate() {
                    let x = start_x + j;
                    if x < width {
                        grid[y][x] = MapDisplay {
                            symbol: ch,
                            color: if i == 0 { Color::Yellow } else { Color::White },
                        };
                    }
                }
            }
        }

        // 添加一個等待動畫點
        let center_x = width / 2;
        let animation_y = center_y + 2;
        if animation_y < height {
            grid[animation_y][center_x] = MapDisplay {
                symbol: '●',
                color: Color::Green,
            };
        }
    }

    /// 創建基礎地圖網格
    fn create_map_grid(
        &self,
        game_state: &GameState,
        viewport: &ViewportManager,
        terminal_width: u16,
        terminal_height: u16,
    ) -> Vec<Vec<MapDisplay>> {
        let width = terminal_width as usize;
        let height = terminal_height as usize;

        // 初始化為空地
        let mut grid = vec![vec![MapDisplay::EMPTY; width]; height];

        grid
    }
    /// 渲染實體
    fn render_entities(
        &self,
        game_state: &GameState,
        grid: &mut Vec<Vec<MapDisplay>>,
        viewport: &ViewportManager,
        terminal_width: u16,
        terminal_height: u16,
    ) {
        let term_width = terminal_width as usize;
        let term_height = terminal_height as usize;
        let player_pos = game_state.local_player.position;

        // 渲染自己的玩家
        if let Some((x, y)) =
            viewport.world_to_screen(player_pos, player_pos, term_width, term_height)
        {
            grid[y][x] = MapDisplay::PLAYER_SELF;
        }

        // 渲染其他玩家
        for (_name, player_state) in &game_state.other_players {
            let pos = Vec2::new(player_state.position.0, player_state.position.1);
            if let Some((x, y)) = viewport.world_to_screen(pos, player_pos, term_width, term_height)
            {
                grid[y][x] = MapDisplay::PLAYER_ENEMY;
            }
        }

        // 渲染己方召喚物
        for summon in &game_state.local_player.summons {
            if let Some((x, y)) =
                viewport.world_to_screen(summon.position, player_pos, term_width, term_height)
            {
                grid[y][x] = MapDisplay::SUMMON_ALLY;
            }
        }

        // 渲染其他實體
        for entity in game_state.entities.values() {
            if let Some((x, y)) =
                viewport.world_to_screen(entity.position, player_pos, term_width, term_height)
            {
                let display = match entity.entity_type {
                    EntityType::Player(_) => MapDisplay::PLAYER_ENEMY,
                    EntityType::Summon(_) => {
                        if entity.owner.as_ref() == Some(&game_state.local_player.name) {
                            MapDisplay::SUMMON_ALLY
                        } else {
                            MapDisplay::SUMMON_ENEMY
                        }
                    }
                    EntityType::Projectile => MapDisplay::PROJECTILE,
                    EntityType::Effect => MapDisplay::EFFECT,
                };
                grid[y][x] = display;
            }
        }
    }

    /// 渲染視野範圍和額外信息
    fn render_vision_range(
        &self,
        grid: &mut Vec<Vec<MapDisplay>>,
        _game_state: &GameState,
        terminal_width: u16,
        terminal_height: u16,
    ) {
        let term_width = terminal_width as usize;
        let term_height = terminal_height as usize;

        // 繪製視圖邊界框
        self.draw_vision_border(grid, term_width, term_height);

        // 顯示距離信息（如果有空間）
        if term_width > 20 && term_height > 10 {
            self.add_distance_markers(grid, term_width, term_height);
        }
    }

    /// 繪製視野邊界框
    fn draw_vision_border(
        &self,
        grid: &mut Vec<Vec<MapDisplay>>,
        term_width: usize,
        term_height: usize,
    ) {
        for x in 0..term_width {
            // 上邊界
            if grid[0][x].symbol == MapDisplay::EMPTY.symbol {
                grid[0][x] = if x == 0 {
                    MapDisplay {
                        symbol: '┌',
                        color: Color::Yellow,
                    }
                } else if x == term_width - 1 {
                    MapDisplay {
                        symbol: '┐',
                        color: Color::Yellow,
                    }
                } else {
                    MapDisplay {
                        symbol: '─',
                        color: Color::Yellow,
                    }
                };
            }
            // 下邊界
            if grid[term_height - 1][x].symbol == MapDisplay::EMPTY.symbol {
                grid[term_height - 1][x] = if x == 0 {
                    MapDisplay {
                        symbol: '└',
                        color: Color::Yellow,
                    }
                } else if x == term_width - 1 {
                    MapDisplay {
                        symbol: '┘',
                        color: Color::Yellow,
                    }
                } else {
                    MapDisplay {
                        symbol: '─',
                        color: Color::Yellow,
                    }
                };
            }
        }

        for y in 1..term_height - 1 {
            // 左邊界
            if grid[y][0].symbol == MapDisplay::EMPTY.symbol {
                grid[y][0] = MapDisplay {
                    symbol: '│',
                    color: Color::Yellow,
                };
            }
            // 右邊界
            if grid[y][term_width - 1].symbol == MapDisplay::EMPTY.symbol {
                grid[y][term_width - 1] = MapDisplay {
                    symbol: '│',
                    color: Color::Yellow,
                };
            }
        }
    }

    /// 添加距離標記
    fn add_distance_markers(
        &self,
        grid: &mut Vec<Vec<MapDisplay>>,
        term_width: usize,
        term_height: usize,
    ) {
        let center_x = term_width / 2;
        let center_y = term_height / 2;

        // 在 1/4 和 3/4 位置添加距離環
        let quarter_x = term_width / 4;
        let three_quarter_x = term_width * 3 / 4;
        let quarter_y = term_height / 4;
        let three_quarter_y = term_height * 3 / 4;

        let distance_marker = MapDisplay {
            symbol: '+',
            color: Color::DarkYellow,
        };

        // 在適當的位置添加距離標記
        if grid[quarter_y][center_x].symbol == MapDisplay::EMPTY.symbol {
            grid[quarter_y][center_x] = distance_marker;
        }
        if grid[three_quarter_y][center_x].symbol == MapDisplay::EMPTY.symbol {
            grid[three_quarter_y][center_x] = distance_marker;
        }
        if grid[center_y][quarter_x].symbol == MapDisplay::EMPTY.symbol {
            grid[center_y][quarter_x] = distance_marker;
        }
        if grid[center_y][three_quarter_x].symbol == MapDisplay::EMPTY.symbol {
            grid[center_y][three_quarter_x] = distance_marker;
        }
    }

    /// 標記視野中心
    fn mark_vision_center(
        &self,
        grid: &mut Vec<Vec<MapDisplay>>,
        term_width: usize,
        term_height: usize,
    ) {
        let center_x = term_width / 2;
        let center_y = term_height / 2;

        // 如果中心位置是空的，添加一個中心標記
        if center_x < term_width
            && center_y < term_height
            && grid[center_y][center_x].symbol == MapDisplay::EMPTY.symbol
        {
            grid[center_y][center_x] = MapDisplay {
                symbol: '·',
                color: Color::White,
            };
        }
    }

    /// 打印地圖到終端
    fn print_map(&self, stdout: &mut io::Stdout, grid: &Vec<Vec<MapDisplay>>) -> io::Result<()> {
        for (row_idx, row) in grid.iter().enumerate() {
            queue!(stdout, cursor::MoveTo(0, row_idx as u16))?;
            for display in row {
                queue!(
                    stdout,
                    SetForegroundColor(display.color),
                    Print(display.symbol)
                )?;
            }
            // 清除到行尾，避免殘留字符
            queue!(stdout, Clear(ClearType::UntilNewLine))?;
        }
        queue!(stdout, ResetColor)?;
        Ok(())
    }

    /// 打印底部日誌
    fn print_logs(
        &self,
        stdout: &mut io::Stdout,
        terminal_width: u16,
        terminal_height: u16,
    ) -> io::Result<()> {
        let terminal_height = terminal_height + 3; // 恢復完整終端高度
        crate::terminal_logger::TerminalLogger::global().render_logs(
            stdout,
            terminal_width,
            terminal_height,
            3, // 使用底部3行顯示日誌
        )?;
        Ok(())
    }
}
