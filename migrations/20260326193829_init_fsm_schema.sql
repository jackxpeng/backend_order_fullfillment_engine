-- migrations/YYYYMMDDHHMMSS_init_fsm_schema.up.sql

CREATE TABLE orders (
    order_id SERIAL PRIMARY KEY,
    state VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE line_items (
    item_id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES orders(order_id) ON DELETE CASCADE,
    item_name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING',
    claimed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_orders_state ON orders(state);
CREATE INDEX idx_line_items_status_claimed ON line_items(status, claimed_at);
CREATE INDEX idx_line_items_order_id ON line_items(order_id);
