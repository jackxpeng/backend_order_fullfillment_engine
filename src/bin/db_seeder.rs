use backend_order_fullfillment_engine::seeder::seed_database;
use sqlx::postgres::PgPoolOptions;
use std::env;

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
