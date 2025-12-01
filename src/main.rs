// main.rs
// Main entry point for Personal Finance Tracker

mod models;
mod seed;

use dotenvy::dotenv;
use sqlx::SqlitePool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Create database connection pool (only if command requires it)
        let pool = match args[1].as_str() {
            "help" | "--help" | "-h" => {
                // Help doesn't need database connection
                print_usage();
                return Ok(());
            }
            _ => {
                println!("📦 Connecting to database...");
                let pool = SqlitePool::connect(&database_url).await?;

                // Enable foreign key constraints (important for SQLite)
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&pool)
                    .await?;

                println!("   ✓ Connected to: {}", database_url);
                println!();
                pool
            }
        };

        match args[1].as_str() {
            "db_seed" => {
                // Run database seeding
                seed::seed_database(&pool).await?;
            }
            "db_clear" => {
                // Clear all data
                println!("⚠️  WARNING: This will delete ALL data!");
                println!("   Press Enter to continue, Ctrl+C to cancel...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                seed::clear_database(&pool).await?;
                println!();
                println!("✅ Database cleared successfully!");
            }
            "db_reseed" => {
                // Clear and re-seed
                println!("🔄 Re-seeding database (clear + seed)...");
                println!();
                seed::clear_database(&pool).await?;
                println!();
                seed::seed_database(&pool).await?;
            }
            "db_status" => {
                // Show database status
                print_database_status(&pool).await?;
            }
            _ => {
                println!("❌ Unknown command: {}", args[1]);
                println!();
                print_usage();
            }
        }

        // Close the connection pool
        pool.close().await;
    } else {
        // No arguments - show help
        print_usage();
    }

    Ok(())
}

/// Print usage information
fn print_usage() {
    println!("┌─────────────────────────────────────────────┐");
    println!("│   Personal Finance Tracker - CLI Tool      │");
    println!("└─────────────────────────────────────────────┘");
    println!();
    println!("Usage: cargo run <command>");
    println!();
    println!("Commands:");
    println!("  db_status     Show database status (record counts)");
    println!("  db_seed       Populate database with sample data");
    println!("  db_clear      Clear all data from database");
    println!("  db_reseed     Clear and re-seed database");
    println!("  help          Show this help message");
    println!();
    println!("Examples:");
    println!("  cargo run db_status   # Check how many records exist");
    println!("  cargo run db_seed     # Add sample data");
    println!("  cargo run db_reseed   # Reset with fresh data");
    println!();
}

/// Print current database status
async fn print_database_status(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("📊 Database Status:");
    println!();

    // Count records in each table
    let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    let accounts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool)
        .await?;
    let categories: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories")
        .fetch_one(pool)
        .await?;
    let transactions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
        .fetch_one(pool)
        .await?;
    let recurring: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM recurring_transactions")
        .fetch_one(pool)
        .await?;
    let rates: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM exchange_rates")
        .fetch_one(pool)
        .await?;

    println!("  Users:                   {}", users);
    println!("  Accounts:                {}", accounts);
    println!("  Categories:              {}", categories);
    println!("  Transactions:            {}", transactions);
    println!("  Recurring Transactions:  {}", recurring);
    println!("  Exchange Rates:          {}", rates);
    println!();

    if users == 0 {
        println!("💡 Tip: Database is empty. Run 'cargo run db_seed' to populate with sample data");
        println!();
    } else {
        println!("✅ Database contains data");
        println!();
    }

    Ok(())
}
