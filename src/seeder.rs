use sqlx::postgres::PgPool;

pub async fn seed_database(pool: &PgPool, orders_count: i32, items_per_order: i32) -> Result<(), sqlx::Error> {
    for _ in 0..orders_count {
        let order_id: i32 = sqlx::query_scalar!(
            "INSERT INTO orders (state) VALUES ('PENDING') RETURNING order_id"
        )
        .fetch_one(pool)
        .await?;

        println!("Created PENDING Order #{}", order_id);

        for item_num in 1..=items_per_order {
            let item_name = format!("Product SKU-{}", item_num);
            
            sqlx::query!(
                "INSERT INTO line_items (order_id, item_name, status) VALUES ($1, $2, 'PENDING')",
                order_id,
                item_name
            )
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_integration_seed_database(pool: PgPool) {
        // We can check how many orders exist before
        let initial_orders: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM orders").fetch_one(&pool).await.unwrap().unwrap_or(0);
        let initial_items: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM line_items").fetch_one(&pool).await.unwrap().unwrap_or(0);

        // Seed
        seed_database(&pool, 2, 2).await.unwrap();

        let final_orders: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM orders").fetch_one(&pool).await.unwrap().unwrap_or(0);
        let final_items: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM line_items").fetch_one(&pool).await.unwrap().unwrap_or(0);

        assert_eq!(final_orders - initial_orders, 2);
        assert_eq!(final_items - initial_items, 4);
    }
}
