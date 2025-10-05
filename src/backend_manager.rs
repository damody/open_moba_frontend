/// 後端程序管理器
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, Context};
use log::{info, warn, error};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

use crate::config::AppConfig;

/// 後端管理器
pub struct BackendManager {
    /// 後端程序句柄
    process: Arc<Mutex<Option<Child>>>,
    /// 配置
    config: AppConfig,
}

impl BackendManager {
    /// 創建新的後端管理器
    pub fn new(config: AppConfig) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            config,
        }
    }
    
    /// 啟動後端程序
    pub async fn start(&self) -> Result<()> {
        let mut process_guard = self.process.lock().await;
        
        // 先清理系統中所有舊的後端進程
        self.cleanup_existing_backend_processes().await?;
        
        // 檢查是否已經在運行
        if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(Some(_)) => {
                    info!("後端程序已退出，重新啟動...");
                },
                Ok(None) => {
                    info!("後端程序已在運行");
                    return Ok(());
                },
                Err(e) => {
                    warn!("無法檢查後端程序狀態: {}", e);
                }
            }
        }
        
        // 取得執行檔路徑
        let exe_path = self.config.get_backend_executable_path()
            .context("無法取得後端執行檔路徑")?;
        
        info!("🚀 啟動後端程序: {:?}", exe_path);
        
        // 準備命令
        let mut cmd = Command::new(&exe_path);
        
        // 添加參數
        for arg in &self.config.backend.args {
            cmd.arg(arg);
        }
        
        // 設定工作目錄
        if let Some(ref work_dir) = self.config.backend.working_directory {
            let work_path = PathBuf::from(work_dir);
            let abs_work_dir = if work_path.is_relative() {
                std::env::current_dir()?.join(work_path)
            } else {
                work_path
            };
            cmd.current_dir(abs_work_dir);
        } else {
            // 預設使用執行檔所在目錄作為工作目錄
            if let Some(parent) = exe_path.parent() {
                cmd.current_dir(parent);
            }
        }
        
        // 設定環境變數
        for (key, value) in &self.config.backend.env {
            cmd.env(key, value);
        }
        
        // 設定輸出重定向到 backend.log
        let log_file = std::fs::File::create("backend.log")
            .context("無法創建 backend.log 文件")?;
        cmd.stdout(log_file.try_clone().context("無法複製 log 文件句柄")?);
        cmd.stderr(log_file);
        
        // 啟動程序
        match cmd.spawn() {
            Ok(child) => {
                info!("✅ 後端程序已啟動 (PID: {:?})", child.id());
                info!("📝 後端輸出已重定向到 backend.log");
                *process_guard = Some(child);
                
                // 等待後端啟動
                let delay_ms = self.config.frontend.backend_start_delay;
                info!("⏳ 等待 {}ms 讓後端完成初始化...", delay_ms);
                sleep(Duration::from_millis(delay_ms)).await;
                
                Ok(())
            },
            Err(e) => {
                error!("❌ 無法啟動後端程序: {}", e);
                Err(e.into())
            }
        }
    }
    
    /// 停止後端程序
    pub async fn stop(&self) -> Result<()> {
        let mut process_guard = self.process.lock().await;
        
        if let Some(mut child) = process_guard.take() {
            info!("🛑 停止後端程序...");
            
            // 嘗試優雅關閉
            match child.kill() {
                Ok(_) => {
                    info!("已發送停止信號");
                    
                    // 等待程序退出
                    let timeout_ms = self.config.frontend.backend_shutdown_timeout;
                    let timeout = Duration::from_millis(timeout_ms);
                    
                    match tokio::time::timeout(timeout, async {
                        loop {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    info!("✅ 後端程序已退出 (狀態: {:?})", status);
                                    return Ok(());
                                },
                                Ok(None) => {
                                    sleep(Duration::from_millis(100)).await;
                                },
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                    }).await {
                        Ok(Ok(_)) => {},
                        Ok(Err(e)) => {
                            warn!("等待後端程序退出時發生錯誤: {}", e);
                        },
                        Err(_) => {
                            warn!("後端程序在 {}ms 內未退出", timeout_ms);
                        }
                    }
                },
                Err(e) => {
                    error!("無法停止後端程序: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            info!("後端程序未在運行");
        }
        
        Ok(())
    }
    
    /// 重啟後端程序
    pub async fn restart(&self) -> Result<()> {
        info!("🔄 重啟後端程序...");
        self.stop().await?;
        self.start().await?;
        Ok(())
    }
    
    /// 檢查後端程序是否運行中
    pub async fn is_running(&self) -> bool {
        let mut process_guard = self.process.lock().await;
        
        if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(None) => true,  // 程序仍在運行
                _ => false,
            }
        } else {
            false
        }
    }
    
    /// 取得程序 ID
    pub async fn get_pid(&self) -> Option<u32> {
        let process_guard = self.process.lock().await;
        process_guard.as_ref().map(|child| child.id())
    }
    
    /// 清理系統中現有的後端進程
    async fn cleanup_existing_backend_processes(&self) -> Result<()> {
        info!("🧹 清理現有的後端進程...");
        
        #[cfg(target_os = "windows")]
        {
            // Windows: 使用 taskkill 命令終止 omobab.exe 進程
            let output = std::process::Command::new("taskkill")
                .args(&["/F", "/IM", "omobab.exe"])
                .output();
                
            match output {
                Ok(result) => {
                    if result.status.success() {
                        info!("✅ 已清理 Windows 上的 omobab.exe 進程");
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        if stderr.contains("not found") || stderr.contains("未找到") {
                            info!("ℹ️  沒有找到需要清理的 omobab.exe 進程");
                        } else {
                            warn!("⚠️  清理進程時出現警告: {}", stderr);
                        }
                    }
                },
                Err(e) => {
                    warn!("❌ 無法執行 taskkill 命令: {}", e);
                }
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Unix/Linux: 使用 pkill 命令
            let output = std::process::Command::new("pkill")
                .args(&["-f", "omobab"])
                .output();
                
            match output {
                Ok(result) => {
                    if result.status.success() {
                        info!("✅ 已清理 Unix 上的 omobab 進程");
                    } else {
                        info!("ℹ️  沒有找到需要清理的 omobab 進程");
                    }
                },
                Err(e) => {
                    // pkill 命令可能不存在，使用 killall 作為備選
                    let output = std::process::Command::new("killall")
                        .args(&["-9", "omobab"])
                        .output();
                        
                    match output {
                        Ok(result) => {
                            if result.status.success() {
                                info!("✅ 已使用 killall 清理 omobab 進程");
                            } else {
                                info!("ℹ️  沒有找到需要清理的 omobab 進程");
                            }
                        },
                        Err(_) => {
                            warn!("❌ 無法執行進程清理命令 (pkill/killall 都不可用): {}", e);
                        }
                    }
                }
            }
        }
        
        // 等待一段時間讓進程完全退出
        sleep(Duration::from_millis(500)).await;
        
        Ok(())
    }
}

impl Drop for BackendManager {
    fn drop(&mut self) {
        // 確保程序在管理器被刪除時停止
        if let Ok(mut process_guard) = self.process.try_lock() {
            if let Some(mut child) = process_guard.take() {
                let _ = child.kill();
                info!("🛑 後端管理器被刪除，停止後端程序 (PID: {:?})", child.id());
            }
        }
        
        // 額外清理：確保所有 omobab 進程都被終止
        info!("🧹 最終清理所有後端進程...");
        
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("taskkill")
                .args(&["/F", "/IM", "omobab.exe"])
                .output();
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            let _ = std::process::Command::new("pkill")
                .args(&["-9", "-f", "omobab"])
                .output();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_backend_manager_lifecycle() {
        // 創建測試配置
        let config = AppConfig::default();
        let manager = BackendManager::new(config);
        
        // 測試啟動
        assert!(!manager.is_running().await);
        
        // 注意：實際測試需要有效的後端執行檔
        // 這裡只測試基本功能
    }
}