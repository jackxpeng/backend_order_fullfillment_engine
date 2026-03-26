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
                
                // TODO: Ticket 4 (Update Parent Order)
                // TODO: Ticket 5 (Simulate Work)
                
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
