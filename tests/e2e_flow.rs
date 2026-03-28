use sqlx::PgPool;
use backend_order_fullfillment_engine::{seeder, worker, aggregation};

#[sqlx::test]
async fn test_full_lifecycle_e2e(pool: PgPool) {
    // 1. Create a raw order via Seeder
    // We create exactly 1 order with exactly 5 items (so one is OUT_OF_STOCK based on simulator mod 5 logic).
    seeder::seed_database(&pool, 1, 5).await.unwrap();
    
    // Grab the order_id we just seeded (there should be only one)
    let order_id: i32 = sqlx::query_scalar!("SELECT order_id FROM orders LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    // The order should immediately be PENDING
    let state: String = sqlx::query_scalar!("SELECT state FROM orders WHERE order_id = $1", order_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(state, "PENDING");

    // 2. Mock the Worker Node Execution
    // Let's loop the worker until process_next_job returns false (no more items)
    let mut jobs_processed = 0;
    while worker::process_next_job(&pool).await.unwrap() {
        jobs_processed += 1;
    }
    
    // We had 5 items, so 5 jobs should be processed
    assert_eq!(jobs_processed, 5);

    // After all items are finalized, the order should still be 'PROCESSING', not 'SHIPPED'
    let state_after_worker: String = sqlx::query_scalar!("SELECT state FROM orders WHERE order_id = $1", order_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(state_after_worker, "PROCESSING");

    // 3. Mock the Aggregation Sweeper Execution
    // Now we run the offline cron job
    let swept = aggregation::process_orders(&pool).await.unwrap();
    assert_eq!(swept, 1); // Exact 1 order should be swept
    
    // Let's see what happened to our items
    let out_of_stock_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM line_items WHERE status = 'OUT_OF_STOCK' AND order_id = $1", order_id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(0);
        
    let packed_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM line_items WHERE status = 'PACKED' AND order_id = $1", order_id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(0);
        
    println!("Packed: {}, OOS: {}", packed_count, out_of_stock_count);
    
    // The simulator turns item_id % 5 == 0 into OUT_OF_STOCK
    // We expect at least one OOS or all packed, so PARTIALLY_SHIPPED or SHIPPED
    let expected_state = if out_of_stock_count > 0 && packed_count > 0 {
        "PARTIALLY_SHIPPED"
    } else if out_of_stock_count > 0 {
        "FAILED"
    } else {
        "SHIPPED"
    };

    // 4. Verify Final E2E State
    let final_state: String = sqlx::query_scalar!("SELECT state FROM orders WHERE order_id = $1", order_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        
    assert_eq!(final_state, expected_state);
}
