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
    
    /// 世界座標轉螢幕座標
    pub fn world_to_screen(
        &self,
        world_pos: Vec2<f32>,
        camera_center: Vec2<f32>,
        screen_width: usize,
        screen_height: usize,
    ) -> Option<(usize, usize)> {
        // 計算視圖邊界
        let view_left = camera_center.x - self.view_width / 2.0;
        let view_top = camera_center.y - self.view_height / 2.0;
        let view_right = camera_center.x + self.view_width / 2.0;
        let view_bottom = camera_center.y + self.view_height / 2.0;
        
        // 檢查是否在視圖範圍內
        if world_pos.x < view_left || world_pos.x > view_right || 
           world_pos.y < view_top || world_pos.y > view_bottom {
            return None;
        }
        
        // 計算螢幕座標
        let relative_x = world_pos.x - view_left;
        let relative_y = world_pos.y - view_top;
        
        let screen_x = ((relative_x / self.view_width) * screen_width as f32) as usize;
        let screen_y = ((relative_y / self.view_height) * screen_height as f32) as usize;
        
        if screen_x < screen_width && screen_y < screen_height {
            Some((screen_x, screen_y))
        } else {
            None
        }
    }
    
    /// 螢幕座標轉世界座標
    pub fn screen_to_world(
        &self,
        screen_x: u16,
        screen_y: u16,
        camera_center: Vec2<f32>,
        screen_width: usize,
        screen_height: usize,
    ) -> Vec2<f32> {
        // 計算相對座標 (0.0 到 1.0)
        let relative_x = screen_x as f32 / screen_width as f32;
        let relative_y = screen_y as f32 / screen_height as f32;
        
        // 計算視圖邊界
        let view_left = camera_center.x - self.view_width / 2.0;
        let view_top = camera_center.y - self.view_height / 2.0;
        
        // 轉換為世界座標
        let world_x = view_left + relative_x * self.view_width;
        let world_y = view_top + relative_y * self.view_height;
        
        Vec2::new(world_x, world_y)
    }
}