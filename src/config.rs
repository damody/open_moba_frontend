/// 配置檔案處理
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};

/// 應用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub backend: BackendConfig,
    pub frontend: FrontendConfig,
}

/// 服務器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub mqtt_host: String,
    pub mqtt_port: u16,
}

/// 後端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub executable_path: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub working_directory: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// 前端配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    pub player_name: String,
    pub hero_type: String,
    pub auto_start_backend: bool,
    pub backend_start_delay: u64,
    pub backend_shutdown_timeout: u64,
    /// 螢幕顯示範圍配置
    pub screen_range: ScreenRangeConfig,
}

/// 螢幕顯示範圍配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenRangeConfig {
    /// 螢幕顯示範圍寬度（遊戲世界單位）
    pub width: f32,
    /// 螢幕顯示範圍高度（遊戲世界單位）
    pub height: f32,
    /// 是否啟用動態範圍調整（根據縮放等調整）
    pub dynamic_range: bool,
    /// 最小顯示範圍（防止過度縮小）
    pub min_width: f32,
    /// 最小顯示範圍（防止過度縮小）
    pub min_height: f32,
    /// 最大顯示範圍（防止過度放大）
    pub max_width: f32,
    /// 最大顯示範圍（防止過度放大）
    pub max_height: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                mqtt_host: "127.0.0.1".to_string(),
                mqtt_port: 1883,
            },
            backend: BackendConfig {
                executable_path: "../omobab/target/debug/omobab".to_string(),
                args: vec![],
                working_directory: None,
                env: HashMap::new(),
            },
            frontend: FrontendConfig {
                player_name: "TestPlayer".to_string(),
                hero_type: "saika_magoichi".to_string(),
                auto_start_backend: true,
                backend_start_delay: 1000,
                backend_shutdown_timeout: 5000,
                screen_range: ScreenRangeConfig {
                    width: 400.0,      // 螢幕顯示範圍寬度（遊戲世界單位）
                    height: 300.0,     // 螢幕顯示範圍高度（遊戲世界單位）
                    dynamic_range: true,
                    min_width: 200.0,  // 最小顯示範圍
                    min_height: 150.0,
                    max_width: 800.0,  // 最大顯示範圍
                    max_height: 600.0,
                },
            },
        }
    }
}

impl AppConfig {
    /// 從檔案載入配置
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("無法讀取配置檔案: {}", path))?;
        
        let config: AppConfig = toml::from_str(&content)
            .with_context(|| format!("無法解析配置檔案: {}", path))?;
        
        Ok(config)
    }
    
    /// 載入配置 (優先使用檔案，否則使用預設值)
    pub fn load() -> Self {
        match Self::from_file("config.toml") {
            Ok(config) => {
                log::info!("已載入配置檔案: config.toml");
                config
            },
            Err(e) => {
                log::warn!("無法載入配置檔案，使用預設值: {}", e);
                Self::default()
            }
        }
    }
    
    /// 儲存配置到檔案
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("無法序列化配置")?;
        
        std::fs::write(path, content)
            .with_context(|| format!("無法寫入配置檔案: {}", path))?;
        
        Ok(())
    }
    
    /// 取得後端執行檔的絕對路徑
    pub fn get_backend_executable_path(&self) -> Result<PathBuf> {
        let path = PathBuf::from(&self.backend.executable_path);
        
        // 如果是相對路徑，轉換為絕對路徑
        let abs_path = if path.is_relative() {
            std::env::current_dir()?.join(path)
        } else {
            path
        };
        
        // 檢查檔案是否存在
        if !abs_path.exists() {
            anyhow::bail!("後端執行檔不存在: {:?}", abs_path);
        }
        
        Ok(abs_path)
    }
    
    /// 根據螢幕解析度獲取顯示範圍配置
    pub fn get_screen_range(&self, screen_width: u32, screen_height: u32) -> ScreenRangeConfig {
        // 根據螢幕解析度調整顯示範圍
        let aspect_ratio = screen_width as f32 / screen_height as f32;
        let base_width = self.frontend.screen_range.width;
        let base_height = self.frontend.screen_range.height;
        
        // 保持寬高比，但根據螢幕大小調整
        let scale_factor = if aspect_ratio > 1.33 {  // 寬螢幕
            (screen_width as f32 / 1920.0).min(1.5)  // 限制最大縮放
        } else {
            (screen_height as f32 / 1080.0).min(1.5)
        };
        
        ScreenRangeConfig {
            width: (base_width * scale_factor).clamp(
                self.frontend.screen_range.min_width,
                self.frontend.screen_range.max_width
            ),
            height: (base_height * scale_factor).clamp(
                self.frontend.screen_range.min_height,
                self.frontend.screen_range.max_height
            ),
            dynamic_range: self.frontend.screen_range.dynamic_range,
            min_width: self.frontend.screen_range.min_width,
            min_height: self.frontend.screen_range.min_height,
            max_width: self.frontend.screen_range.max_width,
            max_height: self.frontend.screen_range.max_height,
        }
    }
}