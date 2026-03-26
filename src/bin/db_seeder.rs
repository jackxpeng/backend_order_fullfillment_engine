use sqlx::postgres::{PgPoolOptions, PgPool};
use std::env;

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

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1. Connect to the database
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://fsm_user:fsm_password@localhost:5432/fulfillment_db".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Connected to DB. Seeding mock orders...");

    seed_database(&pool, 3, 3).await?;

    println!("Seeding complete! Your FSM has data to process.");
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
