use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1. Establish its own connection to the database
    let db_url = "postgres://fsm_user:fsm_password@localhost:5432/fulfillment_db";
    let pool = PgPoolOptions::new()
        .max_connections(2) // Sweeper doesn't need many connections
        .connect(db_url)
        .await?;

    println!("🧹 Standalone Zombie Sweeper daemon started...");
    
    // 2. Run the infinite loop
    loop {
        sleep(Duration::from_secs(10)).await;

        let sweep_result = sqlx::query!(
            r#"
            UPDATE line_items 
            SET status = 'PENDING', claimed_at = NULL 
            WHERE status = 'PICKING' 
            AND claimed_at < NOW() - INTERVAL '5 minutes'
            "#
        )
        .execute(&pool)
        .await;

        match sweep_result {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    println!(
                        "🧟 ZOMBIE SWEEPER: Found {} crashed jobs. Reset to PENDING!", 
                        result.rows_affected()
                    );
                }
            }
            Err(e) => eprintln!("Sweeper error: {}", e),
        }
    }
}
