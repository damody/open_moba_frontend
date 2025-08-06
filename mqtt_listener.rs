// 簡單的 MQTT 監聽器，用於驗證後端是否發送訊息
use rumqttc::{Client, MqttOptions, QoS, Event, Packet};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let mut mqttoptions = MqttOptions::new("mqtt_listener", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_clean_session(true);
    
    let (client, mut connection) = Client::new(mqttoptions, 10);
    
    // 訂閱所有相關主題
    client.subscribe("td/all/res", QoS::AtMostOnce)?;
    client.subscribe("td/+/send", QoS::AtMostOnce)?;
    client.subscribe("ability_test/response", QoS::AtMostOnce)?;
    
    println!("🎧 MQTT 監聽器已啟動，監聽主題:");
    println!("  - td/all/res");
    println!("  - td/+/send");
    println!("  - ability_test/response");
    println!("");
    
    // 主事件循環
    for (i, notification) in connection.iter().enumerate() {
        match notification {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                let topic = &publish.topic;
                let payload = String::from_utf8_lossy(&publish.payload);
                
                println!("📨 收到 MQTT 訊息 #{}", i + 1);
                println!("   主題: {}", topic);
                println!("   內容: {}", payload);
                println!("   時間: {:?}", std::time::SystemTime::now());
                println!("");
            },
            Ok(Event::Incoming(packet)) => {
                println!("📦 收到其他 MQTT 包: {:?}", packet);
            },
            Ok(Event::Outgoing(_)) => {
                // 忽略發送事件
            },
            Err(e) => {
                println!("❌ MQTT 錯誤: {}", e);
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        
        // 運行 30 秒後退出
        if i >= 300 {  // 假設每 100ms 一個事件
            break;
        }
    }
    
    println!("🏁 監聽結束");
    Ok(())
}