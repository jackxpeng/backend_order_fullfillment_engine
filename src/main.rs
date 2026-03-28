use backend_order_fullfillment_engine::worker::process_next_job;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://fsm_user:fsm_password@localhost:5432/fulfillment_db".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Worker Node Started. Polling for PENDING jobs...");

    loop {
        match process_next_job(&pool).await {
            Ok(true) => {
                sleep(Duration::from_millis(500)).await;
            }
            Ok(false) => {
                println!("😴 No pending items found. Sleeping for 3 seconds...");
                sleep(Duration::from_secs(3)).await;
            }
            Err(e) => {
                eprintln!("❌ Error while processing job: {}", e);
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}
