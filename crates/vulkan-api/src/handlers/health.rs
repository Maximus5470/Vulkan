use std::time::SystemTime;

struct Health{
    uptime_seconds: u64,
    redis_connected: bool,
    timestamp: SystemTime 
}

pub async fn handle(){
    
}