use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time::sleep;

// This struct maps perfectly to the RETURNING clause of our SQL query
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

    // The Infinite Worker Loop
    loop {
        // The Magic Query: Lock, Claim, and Return in one atomic step
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
                
                // Ticket 4 (Update Parent Order)
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

                // If rows_affected is 1, this worker was the first one to touch the order!
                if order_update.rows_affected() > 0 {
                    println!("🚀 ORDER #{} transitioned to PROCESSING", item.order_id);
                }
                // ------------------------------------
                
                // Ticket 5 (Simulate Work)
                sleep(Duration::from_millis(500)).await;

                // For now, just sleep a tiny bit so we can watch it run
                sleep(Duration::from_millis(500)).await;
            }
            None => {
                // If query returns None, it means the SKIP LOCKED bypassed everything 
                // and found absolutely 0 'PENDING' items. 
                println!("😴 No pending items found. Sleeping for 3 seconds...");
                sleep(Duration::from_secs(3)).await;
            }
        }
    }
}
