// main.rs
mod api;
mod models;
mod seed;
mod tui;

use actix_web::{middleware, web, App, HttpServer};
use dotenvy::dotenv;
use sqlx::SqlitePool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let args: Vec<String> = env::args().collect();

    // Connect to database
    println!("Connecting to database...");
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;
    println!("Connected to: {}", database_url);

    if args.len() > 1 {
        match args[1].as_str() {
            "tui" => {
                // Launch TUI
                let mut app = tui::App::new(pool.clone());
                app.run().await?;
                return Ok(());
            }
            "serve" => {
                println!("Starting web server...");
                let bind_address =
                    env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

                println!("Server running at http://{}", bind_address);
                println!("API Documentation:");
                println!("   Users:        GET/POST    /users");
                println!("   User:         GET/PUT/DEL /users/{{id}}");
                println!("   Accounts:     GET/POST    /accounts");
                println!("   Account:      GET/PUT/DEL /accounts/{{id}}");
                println!("   Categories:   GET/POST    /categories");
                println!("   Category:     GET/PUT/DEL /categories/{{id}}");
                println!("   Transactions: GET/POST    /transactions");
                println!("   Transaction:  GET/PUT/DEL /transactions/{{id}}");
                println!();

                HttpServer::new(move || {
                    App::new()
                        .app_data(web::Data::new(pool.clone()))
                        .wrap(middleware::Logger::default())
                        .configure(api::configure_routes)
                })
                .bind(&bind_address)?
                .run()
                .await?;

                return Ok(());
            }
            "db_seed" => seed::seed_database(&pool).await?,
            "db_clear" => {
                println!("WARNING: This will delete ALL data!");
                println!("Press Enter to continue, Ctrl+C to cancel...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                seed::clear_database(&pool).await?;
                println!();
                println!("Database cleared successfully!");
            }
            "db_reseed" => {
                println!("Re-seeding database (clear + seed)...");
                println!();
                seed::clear_database(&pool).await?;
                seed::seed_database(&pool).await?;
            }
            "db_status" => print_database_status(&pool).await?,
            _ => {
                println!("Unknown command: {}", args[1]);
                println!();
                print_usage();
            }
        }
    } else {
        print_usage();
    }

    Ok(())
}

fn print_usage() {
    println!("+-----------------------------------------+");
    println!("| Personal Finance Tracker - CLI Tool     |");
    println!("+-----------------------------------------+");
    println!();
    println!("Usage: cargo run [command]");
    println!();
    println!("Commands:");
    println!("  tui         Launch Text User Interface");
    println!("  serve       Start REST API server");
    println!("  db_status   Show database status");
    println!("  db_seed     Populate with sample data");
    println!("  db_clear    Clear all data");
    println!("  db_reseed   Clear and re-seed");
    println!("  help        Show this message");
    println!();
}

async fn print_database_status(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("Database Status:");
    println!();

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

    println!(" Users: {}", users);
    println!(" Accounts: {}", accounts);
    println!(" Categories: {}", categories);
    println!(" Transactions: {}", transactions);
    println!(" Recurring Transactions: {}", recurring);
    println!(" Exchange Rates: {}", rates);
    println!();

    if users == 0 {
        println!("Tip: Database is empty. Run 'cargo run db_seed' to populate with sample data");
        println!();
    } else {
        println!("Database contains data");
        println!();
    }
    println!();

    Ok(())
}
