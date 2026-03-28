use backend_order_fullfillment_engine::aggregation::process_orders;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://fsm_user:fsm_password@localhost:5432/fulfillment_db".to_string());
        
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await?;

    println!("📊 Aggregation Sweeper daemon started...");

    loop {
        sleep(Duration::from_secs(5)).await;

        match process_orders(&pool).await {
            Ok(count) if count > 0 => {
                // Log only if there were items to keep standard out clean
            }
            Err(e) => {
                eprintln!("❌ Error while processing orders: {}", e);
            }
            _ => {}
        }
    }
}
