use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1. Connect to the database
    let db_url = "postgres://fsm_user:fsm_password@localhost:5432/fulfillment_db";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    println!("Connected to DB. Seeding mock orders...");

    // 2. Create 3 mock orders
    for _ in 0..3 {
        // Insert a parent order and grab its auto-generated ID
        let order_id: i32 = sqlx::query_scalar!(
            "INSERT INTO orders (state) VALUES ('PENDING') RETURNING order_id"
        )
        .fetch_one(&pool)
        .await?;

        println!("Created PENDING Order #{}", order_id);

        // 3. Attach 3 line items to this specific order
        for item_num in 1..=3 {
            let item_name = format!("Product SKU-{}", item_num);
            
            sqlx::query!(
                "INSERT INTO line_items (order_id, item_name, status) VALUES ($1, $2, 'PENDING')",
                order_id,
                item_name
            )
            .execute(&pool)
            .await?;
        }
    }

    println!("Seeding complete! Your FSM has data to process.");
    Ok(())
}

