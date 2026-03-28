use backend_order_fullfillment_engine::zombie::sweep_zombies;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1. Establish its own connection to the database
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://fsm_user:fsm_password@localhost:5432/fulfillment_db".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(2) // Sweeper doesn't need many connections
        .connect(&db_url)
        .await?;

    println!("🧹 Standalone Zombie Sweeper daemon started...");
    
    // 2. Run the infinite loop
    loop {
        sleep(Duration::from_secs(10)).await;

        match sweep_zombies(&pool).await {
            Ok(count) if count > 0 => {
                println!(
                    "🧟 ZOMBIE SWEEPER: Found {} crashed jobs. Reset to PENDING!", 
                    count
                );
            }
            Ok(_) => {}
            Err(e) => eprintln!("Sweeper error: {}", e),
        }
    }
}
