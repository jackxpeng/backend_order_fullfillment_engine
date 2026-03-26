mod simulator;

use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
struct ClaimedItem {
    item_id: i32,
    order_id: i32,
    item_name: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let db_url = "postgres://fsm_user:fsm_password@localhost:5432/fulfillment_db";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    println!("Worker Node Started. Polling for PENDING jobs...");

    loop {
        let claimed = sqlx::query_as!(
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
        .fetch_optional(&pool)
        .await?;

        match claimed {
            Some(item) => {
                println!(
                    "📦 CLAIMED: Item #{} ({}) for Order #{}",
                    item.item_id, item.item_name, item.order_id
                );
                
                // --- TICKET 4: The Parent Trigger ---
                let order_update = sqlx::query!(
                    "UPDATE orders SET state = 'PROCESSING' WHERE order_id = $1 AND state = 'PENDING'",
                    item.order_id
                )
                .execute(&pool)
                .await?;

                if order_update.rows_affected() > 0 {
                    println!("🚀 ORDER #{} transitioned to PROCESSING", item.order_id);
                }

                // --- TICKET 5: Simulate Work & Finalize Item ---
                // Delegate the heavy lifting to our simulator module
                simulator::process_item(&pool, item.item_id).await?;                
                
                sleep(Duration::from_millis(500)).await;
            }
            None => {
                println!("😴 No pending items found. Sleeping for 3 seconds...");
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}
