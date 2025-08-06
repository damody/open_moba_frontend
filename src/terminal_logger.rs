/// 終端視圖專用日誌系統
/// 
/// 在視圖模式下收集日誌並顯示在底部區域
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use crossterm::{
    cursor,
    queue,
    style::{Color, Print, SetForegroundColor, ResetColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

/// 日誌條目
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub timestamp: std::time::Instant,
}

/// 終端日誌收集器
pub struct TerminalLogger {
    entries: Arc<Mutex<VecDeque<LogEntry>>>,
    max_entries: usize,
}

impl TerminalLogger {
    /// 創建新的終端日誌收集器
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
            max_entries,
        }
    }
    
    /// 獲取全局實例
    pub fn global() -> &'static TerminalLogger {
        static mut LOGGER: Option<TerminalLogger> = None;
        static INIT: std::sync::Once = std::sync::Once::new();
        
        INIT.call_once(|| {
            unsafe {
                LOGGER = Some(TerminalLogger::new(100));
            }
        });
        
        unsafe { LOGGER.as_ref().unwrap() }
    }
    
    /// 添加日誌條目
    pub fn log(&self, level: &str, message: String) {
        let mut entries = self.entries.lock().unwrap();
        entries.push_back(LogEntry {
            level: level.to_string(),
            message,
            timestamp: std::time::Instant::now(),
        });
        
        // 限制最大條目數
        while entries.len() > self.max_entries {
            entries.pop_front();
        }
    }
    
    /// 獲取最近的日誌條目
    pub fn get_recent_logs(&self, count: usize) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries.iter()
            .rev()
            .take(count)
            .rev()
            .cloned()
            .collect()
    }
    
    /// 清空日誌
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }
    
    /// 在終端底部渲染日誌
    pub fn render_logs(&self, stdout: &mut io::Stdout, terminal_width: u16, terminal_height: u16, log_lines: usize) -> io::Result<()> {
        let logs = self.get_recent_logs(log_lines);
        let log_start_y = terminal_height.saturating_sub(log_lines as u16);
        
        // 清空日誌區域
        for i in 0..log_lines {
            queue!(stdout, cursor::MoveTo(0, log_start_y + i as u16))?;
            queue!(stdout, Clear(ClearType::CurrentLine))?;
        }
        
        // 渲染日誌
        for (i, entry) in logs.iter().enumerate() {
            if i >= log_lines {
                break;
            }
            
            queue!(stdout, cursor::MoveTo(0, log_start_y + i as u16))?;
            
            // 設置顏色
            let color = match entry.level.as_str() {
                "ERROR" => Color::Red,
                "WARN" => Color::Yellow,
                "INFO" => Color::Green,
                "DEBUG" => Color::Blue,
                _ => Color::White,
            };
            
            queue!(stdout, SetForegroundColor(color))?;
            queue!(stdout, Print(format!("[{}]", entry.level)))?;
            queue!(stdout, SetForegroundColor(Color::White))?;
            
            // 截斷過長的訊息
            let max_msg_len = terminal_width as usize - 8; // 留出級別標籤空間
            let message = if entry.message.len() > max_msg_len {
                format!("{}...", &entry.message[..max_msg_len - 3])
            } else {
                entry.message.clone()
            };
            
            queue!(stdout, Print(format!(" {}", message)))?;
        }
        
        queue!(stdout, ResetColor)?;
        Ok(())
    }
}

/// 自定義日誌寫入器，將日誌重定向到終端日誌收集器
pub struct TerminalLogWriter;

impl Write for TerminalLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Ok(message) = std::str::from_utf8(buf) {
            // 解析日誌級別和訊息
            let trimmed = message.trim();
            if !trimmed.is_empty() {
                // 簡單解析 env_logger 格式
                let level = if trimmed.contains("[ERROR]") {
                    "ERROR"
                } else if trimmed.contains("[WARN]") {
                    "WARN"
                } else if trimmed.contains("[INFO]") {
                    "INFO"
                } else if trimmed.contains("[DEBUG]") {
                    "DEBUG"
                } else {
                    "INFO"
                };
                
                // 移除時間戳和其他格式化內容，只保留實際訊息
                let msg = trimmed
                    .split("] ")
                    .last()
                    .unwrap_or(trimmed)
                    .to_string();
                
                TerminalLogger::global().log(level, msg);
            }
        }
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}