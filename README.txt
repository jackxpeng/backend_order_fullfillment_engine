📦 Backend Order Fulfillment Engine
A robust, distributed Finite State Machine (FSM) built with Rust and PostgreSQL. This engine manages the lifecycle of warehouse orders using high-concurrency patterns to ensure no two workers ever process the same item.

🏗 System Architecture
The system follows a Parent-Child FSM pattern. An Order (Parent) consists of multiple LineItems (Children). The state of the Parent is derived from the collective states of its Children.

State Transitions
Orders: PENDING → PROCESSING → SHIPPED | PARTIALLY_SHIPPED | FAILED

Line Items: PENDING → PICKING → PACKED | OUT_OF_STOCK

Core Components
The Worker (main.rs): The primary engine. It claims PENDING line items using PostgreSQL's FOR UPDATE SKIP LOCKED for atomic, thread-safe job acquisition.

Zombie Sweeper (bin/zombie_sweeper.rs): A cleanup microservice that detects "zombie" items (stuck in PICKING for >5 minutes due to worker crashes) and resets them to PENDING.

Aggregation Sweeper (bin/aggregation_sweeper.rs): A reactive microservice that monitors PROCESSING orders and finalizes them once all child items reach a terminal state.

Database Seeder (bin/db_seeder.rs): A utility to inject mock order data for testing.

📂 Code Structure
Plaintext
.
├── migrations/             # SQLx database schema migrations
├── src/
│   ├── bin/                # Standalone microservices
│   │   ├── aggregation_sweeper.rs
│   │   ├── db_seeder.rs
│   │   └── zombie_sweeper.rs
│   ├── main.rs             # Main Worker & Orchestrator
│   └── simulator.rs        # Logic for warehouse work simulation
├── docker-compose.yml      # Local PostgreSQL environment
└── Cargo.toml              # Rust dependencies (SQLx, Tokio, etc.)
🚀 The Final Show (How to Run)
1. Spin up the Database
Ensure Docker is running and execute:

Bash
docker compose up -d
sqlx migrate run
2. Seed the Data
Inject fresh pending orders into the system:

Bash
cargo run --bin db_seeder
3. Start the Engine
Open three separate terminal tabs to watch the distributed system in action:

Tab 1 (The Worker): cargo run --bin backend_order_fullfillment_engine

Tab 2 (Zombie Sweeper): cargo run --bin zombie_sweeper

Tab 3 (Aggregation Sweeper): cargo run --bin aggregation_sweeper

🛠 Maintenance & Debugging
The "Hard Reset"
If you want to clear the system and re-run the entire fulfillment flow from scratch without deleting your data, run the following SQL in your database client (like DBeaver):

SQL
-- Reset all Parent Orders to PENDING
UPDATE orders SET state = 'PENDING';

-- Reset all Child Items to PENDING and clear ownership timestamps
UPDATE line_items SET status = 'PENDING', claimed_at = NULL;
Tech Stack
Language: Rust

Runtime: Tokio (Async)

Database: PostgreSQL 16

SQL Layer: SQLx (Compile-time verified SQL)
