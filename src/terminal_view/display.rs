/// 地圖顯示符號和顏色定義
use crossterm::style::Color;

/// 地圖符號和顏色定義
#[derive(Clone, Copy)]
pub struct MapDisplay {
    pub symbol: char,
    pub color: Color,
}

impl MapDisplay {
    // 玩家相關符號
    pub const PLAYER_SELF: MapDisplay = MapDisplay { symbol: '@', color: Color::Yellow };
    pub const PLAYER_ALLY: MapDisplay = MapDisplay { symbol: 'A', color: Color::Green };
    pub const PLAYER_ENEMY: MapDisplay = MapDisplay { symbol: 'E', color: Color::Red };
    
    // 單位符號
    pub const SUMMON_ALLY: MapDisplay = MapDisplay { symbol: 's', color: Color::Cyan };
    pub const SUMMON_ENEMY: MapDisplay = MapDisplay { symbol: 'S', color: Color::Magenta };
    pub const PROJECTILE: MapDisplay = MapDisplay { symbol: '*', color: Color::White };
    
    // 地形符號
    pub const EMPTY: MapDisplay = MapDisplay { symbol: '.', color: Color::DarkGrey };
    pub const WALL: MapDisplay = MapDisplay { symbol: '#', color: Color::Grey };
    pub const TREE: MapDisplay = MapDisplay { symbol: 'T', color: Color::DarkGreen };
    pub const WATER: MapDisplay = MapDisplay { symbol: '~', color: Color::Blue };
    pub const MOUNTAIN: MapDisplay = MapDisplay { symbol: '^', color: Color::DarkGrey };
    
    // 視野相關
    pub const VISION_EDGE: MapDisplay = MapDisplay { symbol: '○', color: Color::Yellow };
    pub const FOG_OF_WAR: MapDisplay = MapDisplay { symbol: '?', color: Color::DarkGrey };
    
    // 特效符號
    pub const EFFECT: MapDisplay = MapDisplay { symbol: '!', color: Color::Red };
    pub const EXPLOSION: MapDisplay = MapDisplay { symbol: '%', color: Color::Red };
}