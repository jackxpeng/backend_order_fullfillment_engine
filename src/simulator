use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;

pub async fn process_item(pool: &PgPool, item_id: i32) -> Result<(), sqlx::Error> {
    // 1. Simulate the time it takes to find the item on a shelf
    sleep(Duration::from_millis(1500)).await;

    // 2. Decide the final state (Every 5th item is out of stock)
    let final_status = if item_id % 5 == 0 {
        "OUT_OF_STOCK"
    } else {
        "PACKED"
    };

    // 3. Update the database to the terminal state
    sqlx::query!(
        "UPDATE line_items SET status = $1 WHERE item_id = $2",
        final_status,
        item_id
    )
    .execute(pool)
    .await?;

    println!("✅ FINISHED: Item #{} is now {}", item_id, final_status);
    Ok(())
}
