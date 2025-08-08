/// 視口管理模塊
use vek::Vec2;

/// 視口管理器
pub struct ViewportManager {
    /// 視圖寬度（世界單位）
    pub view_width: f32,
    /// 視圖高度（世界單位）
    pub view_height: f32,
}

impl ViewportManager {
    /// 創建新的視口管理器
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            view_width: width,
            view_height: height,
        }
    }
    
    /// 世界座標轉螢幕座標 (每個字符代表10x10的世界單位)
    pub fn world_to_screen(
        &self,
        world_pos: Vec2<f32>,
        camera_center: Vec2<f32>,
        screen_width: usize,
        screen_height: usize,
    ) -> Option<(usize, usize)> {
        // 每個螢幕字符代表10x10的世界單位
        const WORLD_UNITS_PER_CHAR: f32 = 10.0;
        
        // 計算螢幕中心
        let screen_center_x = screen_width as f32 / 2.0;
        let screen_center_y = screen_height as f32 / 2.0;
        
        // 計算物體相對於相機中心的偏移
        let offset_x = world_pos.x - camera_center.x;
        let offset_y = world_pos.y - camera_center.y;
        
        // 將偏移轉換為螢幕座標
        let screen_x = screen_center_x + (offset_x / WORLD_UNITS_PER_CHAR);
        let screen_y = screen_center_y + (offset_y / WORLD_UNITS_PER_CHAR);
        
        // 轉換為整數座標
        let screen_x = screen_x as isize;
        let screen_y = screen_y as isize;
        
        // 檢查是否在螢幕範圍內
        if screen_x >= 0 && screen_x < screen_width as isize && 
           screen_y >= 0 && screen_y < screen_height as isize {
            Some((screen_x as usize, screen_y as usize))
        } else {
            None
        }
    }
    
    /// 螢幕座標轉世界座標 (每個字符代表10x10的世界單位)
    pub fn screen_to_world(
        &self,
        screen_x: u16,
        screen_y: u16,
        camera_center: Vec2<f32>,
        screen_width: usize,
        screen_height: usize,
    ) -> Vec2<f32> {
        // 每個螢幕字符代表10x10的世界單位
        const WORLD_UNITS_PER_CHAR: f32 = 10.0;
        
        // 計算螢幕中心
        let screen_center_x = screen_width as f32 / 2.0;
        let screen_center_y = screen_height as f32 / 2.0;
        
        // 計算相對於螢幕中心的偏移
        let offset_x = (screen_x as f32 - screen_center_x) * WORLD_UNITS_PER_CHAR;
        let offset_y = (screen_y as f32 - screen_center_y) * WORLD_UNITS_PER_CHAR;
        
        // 轉換為世界座標
        let world_x = camera_center.x + offset_x;
        let world_y = camera_center.y + offset_y;
        
        Vec2::new(world_x, world_y)
    }
}