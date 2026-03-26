use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let db_url = "postgres://fsm_user:fsm_password@localhost:5432/fulfillment_db";
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(db_url)
        .await?;

    println!("📊 Aggregation Sweeper daemon started...");

    loop {
        sleep(Duration::from_secs(5)).await;

        // The Magic Postgres Aggregation Query
        // We only check orders that are currently 'PROCESSING'
        let processing_orders = sqlx::query!(
            r#"
            SELECT 
                o.order_id,
                COUNT(li.item_id) as "total_items!",
                COUNT(li.item_id) FILTER (WHERE li.status = 'PACKED') as "packed_count!",
                COUNT(li.item_id) FILTER (WHERE li.status = 'OUT_OF_STOCK') as "out_of_stock_count!",
                COUNT(li.item_id) FILTER (WHERE li.status IN ('PENDING', 'PICKING')) as "incomplete_count!"
            FROM orders o
            JOIN line_items li ON o.order_id = li.order_id
            WHERE o.state = 'PROCESSING'
            GROUP BY o.order_id
            "#
        )
        .fetch_all(&pool)
        .await?;

        for order in processing_orders {
            // If there are still incomplete items, skip this order and check again later
            if order.incomplete_count > 0 {
                continue; 
            }

            // Determine the final Order state based on the children
            let final_state = if order.packed_count == order.total_items {
                "SHIPPED"
            } else if order.out_of_stock_count == order.total_items {
                "FAILED"
            } else {
                "PARTIALLY_SHIPPED"
            };

            // Transition the parent order
            sqlx::query!(
                "UPDATE orders SET state = $1 WHERE order_id = $2",
                final_state,
                order.order_id
            )
            .execute(&pool)
            .await?;

            println!(
                "🎯 ORDER FINALIZED: Order #{} is now {} (Packed: {}, OOS: {})", 
                order.order_id, final_state, order.packed_count, order.out_of_stock_count
            );
        }
    }
}
