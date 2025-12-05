// main.rs
mod api;
mod exchange_scraper;
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
                println!("   Exchange Rates:   GET/POST    /exchange-rates");
                println!("   Exchange Rate:    GET/PUT/DEL /exchange-rates/{{id}}");
                println!(
                    "   Latest Rates:     GET         /exchange-rates/latest/{{from_currency}}"
                );
                println!("   Convert Currency: GET         /exchange-rates/convert?from={{from}}&to={{to}}&amount={{amount}}");
                println!("   Bulk Delete:      DELETE      /exchange-rates/bulk?from_currency={{currency}}&date={{date}}&source={{source}}");

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
            "scrape_rates" => {
                scrape_exchange_rates(&pool, &args).await?;
            }
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
    println!("  tui                 Launch Text User Interface");
    println!("  serve               Start REST API server");
    println!("  db_status           Show database status");
    println!("  db_seed             Populate with sample data");
    println!(
        "  scrape_rates        Scrape latest FX rates for default currencies - CAD, USD, GBP, EUR"
    );
    println!("  scrape_rates XXX    Scrape latest FX rates for the specific currency code XXX");
    println!("  db_clear            Clear all data");
    println!("  db_reseed           Clear and re-seed");
    println!("  help                Show this message");
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

async fn scrape_exchange_rates(
    pool: &SqlitePool,
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    use exchange_scraper::{print_exchange_rates, ExchangeRateScraper};

    println!("\nStart Foreign Exchange Rates Scraper...");
    println!();

    let scraper = ExchangeRateScraper::new();

    let currencies: Vec<&str> = if args.len() > 2 {
        vec![args[2].as_str()]
    } else {
        vec!["CAD", "USD", "EUR", "GBP"]
    };

    println!(
        "Will scrape the following currencies' FX rates: {:?}",
        currencies
    );
    println!();

    let all_results = scraper.smart_fetch_multiple(pool, currencies).await;

    if all_results.is_empty() {
        println!("Failure: Cannot scrape any currencies' FX Rates.");
        return Ok(());
    }

    println!("\n{}", "=".repeat(50));
    println!("Scraping Results:");
    println!("{}", "=".repeat(50));

    let mut total_saved = 0;
    let mut total_skipped = 0;

    for (currency, (rates, was_up_to_date)) in &all_results {
        println!("\nExchange rate of {}:", currency);
        if *was_up_to_date {
            println!("\n {} has the latest rates, skipped", currency);
            total_skipped += 1;
        } else {
            print_exchange_rates(rates);
            match scraper.save_to_database(pool, rates).await {
                Ok(count) => {
                    println!("\nSave {} exchange rates into db.", count);
                    total_saved += count;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("Scraping completed！");
    println!("   • Currencies skipped: {} in total", total_skipped);
    println!(
        "   • Currencies scraped: {} in total",
        all_results.len() - total_skipped
    );
    println!("   • FX rates added: {} in total", total_saved);
    println!("{}\n", "=".repeat(50));

    Ok(())
}
