use sqlx::postgres::PgPool;

/// Determine the final order state based on the children counts
pub fn determine_final_state(total_items: i64, packed_count: i64, out_of_stock_count: i64) -> &'static str {
    if total_items == 0 || packed_count == total_items {
        "SHIPPED"
    } else if out_of_stock_count == total_items {
        "FAILED"
    } else {
        "PARTIALLY_SHIPPED"
    }
}

pub async fn process_orders(pool: &PgPool) -> Result<usize, sqlx::Error> {
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
    .fetch_all(pool)
    .await?;

    let mut processed_count = 0;

    for order in processing_orders {
        // If there are still incomplete items, skip this order and check again later
        if order.incomplete_count > 0 {
            continue; 
        }

        // Determine the final Order state based on the children
        let final_state = determine_final_state(
            order.total_items,
            order.packed_count,
            order.out_of_stock_count,
        );

        // Transition the parent order
        sqlx::query!(
            "UPDATE orders SET state = $1 WHERE order_id = $2",
            final_state,
            order.order_id
        )
        .execute(pool)
        .await?;

        println!(
            "🎯 ORDER FINALIZED: Order #{} is now {} (Packed: {}, OOS: {})", 
            order.order_id, final_state, order.packed_count, order.out_of_stock_count
        );
        processed_count += 1;
    }

    Ok(processed_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_final_state_shipped() {
        assert_eq!(determine_final_state(5, 5, 0), "SHIPPED");
    }

    #[test]
    fn test_determine_final_state_failed() {
        assert_eq!(determine_final_state(5, 0, 5), "FAILED");
    }

    #[test]
    fn test_determine_final_state_partially_shipped() {
        assert_eq!(determine_final_state(5, 3, 2), "PARTIALLY_SHIPPED");
        assert_eq!(determine_final_state(5, 0, 0), "PARTIALLY_SHIPPED"); 
    }
    
    #[test]
    fn test_determine_final_state_zero_items() {
        assert_eq!(determine_final_state(0, 0, 0), "SHIPPED"); 
    }
    
    #[sqlx::test]
    async fn test_integration_process_orders(pool: PgPool) {
        let order_id: i32 = sqlx::query_scalar!(
            "INSERT INTO orders (state) VALUES ('PROCESSING') RETURNING order_id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        sqlx::query!(
            "INSERT INTO line_items (order_id, item_name, status) VALUES ($1, 'Test Item 1', 'PACKED')",
            order_id
        )
        .execute(&pool)
        .await
        .unwrap();
        
        sqlx::query!(
            "INSERT INTO line_items (order_id, item_name, status) VALUES ($1, 'Test Item 2', 'OUT_OF_STOCK')",
            order_id
        )
        .execute(&pool)
        .await
        .unwrap();
        
        let result = process_orders(&pool).await;
        assert!(result.is_ok(), "Process orders should not fail");
        
        let final_state: String = sqlx::query_scalar!(
            "SELECT state FROM orders WHERE order_id = $1",
            order_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(final_state, "PARTIALLY_SHIPPED");
    }
    
    #[sqlx::test]
    async fn test_integration_incomplete(pool: PgPool) {
        let order_id: i32 = sqlx::query_scalar!(
            "INSERT INTO orders (state) VALUES ('PROCESSING') RETURNING order_id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        sqlx::query!(
            "INSERT INTO line_items (order_id, item_name, status) VALUES ($1, 'Item 1', 'PICKING')",
            order_id
        )
        .execute(&pool)
        .await
        .unwrap();
        
        // Should not transition because it has incomplete items
        process_orders(&pool).await.unwrap();
        
        let final_state: String = sqlx::query_scalar!(
            "SELECT state FROM orders WHERE order_id = $1",
            order_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(final_state, "PROCESSING");
    }
}
