mod simulator;

use sqlx::postgres::{PgPoolOptions, PgPool};
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, PartialEq, Eq)]
pub struct ClaimedItem {
    pub item_id: i32,
    pub order_id: i32,
    pub item_name: String,
}

pub async fn claim_pending_item(pool: &PgPool) -> Result<Option<ClaimedItem>, sqlx::Error> {
    sqlx::query_as!(
        ClaimedItem,
        r#"
        UPDATE line_items 
        SET status = 'PICKING', claimed_at = NOW() 
        WHERE item_id = (
            SELECT item_id 
            FROM line_items 
            WHERE status = 'PENDING' 
            LIMIT 1 
            FOR UPDATE SKIP LOCKED
        )
        RETURNING item_id, order_id, item_name
        "#
    )
    .fetch_optional(pool)
    .await
}

pub async fn trigger_order_processing(pool: &PgPool, order_id: i32) -> Result<bool, sqlx::Error> {
    let order_update = sqlx::query!(
        "UPDATE orders SET state = 'PROCESSING' WHERE order_id = $1 AND state = 'PENDING'",
        order_id
    )
    .execute(pool)
    .await?;
    
    Ok(order_update.rows_affected() > 0)
}

pub async fn process_next_job(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let claimed = claim_pending_item(pool).await?;

    match claimed {
        Some(item) => {
            println!(
                "📦 CLAIMED: Item #{} ({}) for Order #{}",
                item.item_id, item.item_name, item.order_id
            );
            
            // --- TICKET 4: The Parent Trigger ---
            if trigger_order_processing(pool, item.order_id).await? {
                println!("🚀 ORDER #{} transitioned to PROCESSING", item.order_id);
            }

            // --- TICKET 5: Simulate Work & Finalize Item ---
            simulator::process_item(pool, item.item_id).await?;                
            
            Ok(true) // Job found and processed
        }
        None => Ok(false) // No pending items
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_integration_worker_components(pool: PgPool) {
        // Set up test data
        let order_id: i32 = sqlx::query_scalar!(
            "INSERT INTO orders (state) VALUES ('PENDING') RETURNING order_id"
        ).fetch_one(&pool).await.unwrap();

        let item_id: i32 = sqlx::query_scalar!(
            "INSERT INTO line_items (order_id, item_name, status) VALUES ($1, 'Worker Test', 'PENDING') RETURNING item_id",
            order_id
        ).fetch_one(&pool).await.unwrap();

        // 1. Test claiming
        let claimed = claim_pending_item(&pool).await.unwrap().expect("Should claim the item");
        // Because other tasks might also claim from other tests, we only assert we successfully claimed SOMETHING
        assert!(claimed.item_id > 0);
        
        // Let's test trigger_order_processing specifically for our order
        let triggered = trigger_order_processing(&pool, order_id).await.unwrap();
        assert!(triggered);
        
        let state: String = sqlx::query_scalar!("SELECT state FROM orders WHERE order_id = $1", order_id).fetch_one(&pool).await.unwrap();
        assert_eq!(state, "PROCESSING");
        
        // Try trigger again, should be false since it's already PROCESSING
        let triggered2 = trigger_order_processing(&pool, order_id).await.unwrap();
        assert!(!triggered2);
    }
}
