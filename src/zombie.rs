use sqlx::postgres::PgPool;

pub async fn sweep_zombies(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let sweep_result = sqlx::query!(
        r#"
        UPDATE line_items 
        SET status = 'PENDING', claimed_at = NULL 
        WHERE status = 'PICKING' 
        AND claimed_at < NOW() - INTERVAL '5 minutes'
        "#
    )
    .execute(pool)
    .await?;

    Ok(sweep_result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_integration_sweep_zombies(pool: PgPool) {
        let order_id: i32 = sqlx::query_scalar!(
            "INSERT INTO orders (state) VALUES ('PROCESSING') RETURNING order_id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Insert fresh picking item
        let fresh_item_id: i32 = sqlx::query_scalar!(
            "INSERT INTO line_items (order_id, item_name, status, claimed_at) VALUES ($1, 'Fresh', 'PICKING', NOW()) RETURNING item_id",
            order_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // Insert zombie item (claimed 10 minutes ago)
        let zombie_item_id: i32 = sqlx::query_scalar!(
            "INSERT INTO line_items (order_id, item_name, status, claimed_at) VALUES ($1, 'Zombie', 'PICKING', NOW() - INTERVAL '10 minutes') RETURNING item_id",
            order_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let count = sweep_zombies(&pool).await.unwrap();
        assert!(count >= 1); // Should reset at least the zombie item we created

        let fresh_status: String = sqlx::query_scalar!("SELECT status FROM line_items WHERE item_id = $1", fresh_item_id).fetch_one(&pool).await.unwrap();
        let zombie_status: String = sqlx::query_scalar!("SELECT status FROM line_items WHERE item_id = $1", zombie_item_id).fetch_one(&pool).await.unwrap();

        assert_eq!(fresh_status, "PICKING");
        assert_eq!(zombie_status, "PENDING"); // Zombie was reset
    }
}
