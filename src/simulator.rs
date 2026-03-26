use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;

pub async fn process_item(pool: &PgPool, item_id: i32) -> Result<(), sqlx::Error> {
    // 1. Simulate the time it takes to find the item on a shelf
    // We keep this to a small amount in test or mock it, but for our simple tests, we can just let it sleep
    sleep(Duration::from_millis(500)).await;

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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_process_item(pool: PgPool) {
        // We don't need to connect to the DB manually or clean up; 
        // sqlx::test provides a fully migrated, isolated database pool!

        // Insert order and item
        let order_id: i32 = sqlx::query_scalar!(
            "INSERT INTO orders (state) VALUES ('PROCESSING') RETURNING order_id"
        ).fetch_one(&pool).await.unwrap();

        let item_id: i32 = sqlx::query_scalar!(
            "INSERT INTO line_items (order_id, item_name, status) VALUES ($1, 'Test', 'PICKING') RETURNING item_id",
            order_id
        ).fetch_one(&pool).await.unwrap();

        // Run the process
        process_item(&pool, item_id).await.unwrap();

        let final_status: String = sqlx::query_scalar!(
            "SELECT status FROM line_items WHERE item_id = $1", item_id
        ).fetch_one(&pool).await.unwrap();

        let expected_status = if item_id % 5 == 0 { "OUT_OF_STOCK" } else { "PACKED" };
        assert_eq!(final_status, expected_status);
        
        // No cleanup necessary! The temporary database is destroyed when the test finishes.
    }
}
