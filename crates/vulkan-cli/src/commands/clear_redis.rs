use std::{env, error::Error};

use dotenvy;
use redis::{Client, cmd};

pub fn handle() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let redis_client = Client::open(redis_url)?;
    let mut redis_connection = redis_client.get_connection().unwrap();
    cmd("FLUSHDB").query::<()>(&mut redis_connection)?;
    println!("Redis cache cleared successfully.");
    Ok(())
}