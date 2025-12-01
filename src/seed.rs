// seed.rs
// Database seeding module for Personal Finance Tracker
// Populates the database with sample data for development and testing

use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;

/// Main seeding function - populates all tables with sample data
pub async fn seed_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if database is already seeded
    if is_database_seeded(pool).await? {
        println!("⚠️  Database already contains data. Skipping seeding.");
        println!("   To re-seed, delete all data first.");
        return Ok(());
    }

    println!("🌱 Starting database seeding...");
    println!();

    // Seed in order due to foreign key constraints
    seed_users(pool).await?;
    seed_categories(pool).await?;
    seed_accounts(pool).await?;
    seed_transactions(pool).await?;
    seed_transaction_categories(pool).await?;
    seed_recurring_transactions(pool).await?;
    seed_exchange_rates(pool).await?;

    println!();
    println!("✅ Database seeding completed successfully!");
    println!();

    // Print summary
    print_seed_summary(pool).await?;

    Ok(())
}

/// Check if database already has data
async fn is_database_seeded(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count > 0)
}

/// Seed users table
async fn seed_users(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("👤 Seeding users...");

    let users = vec![
        ("alice_wang", "alice@example.com", "$argon2id$v=19$m=19456,t=2,p=1$VE3VyJmJqKmZmZmZmZmZmQ$Jmw/A8cPvgLKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGK"),
        ("bob_chen", "bob@example.com", "$argon2id$v=19$m=19456,t=2,p=1$VE3VyJmJqKmZmZmZmZmZmQ$Jmw/A8cPvgLKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGK"),
        ("carol_liu", "carol@example.com", "$argon2id$v=19$m=19456,t=2,p=1$VE3VyJmJqKmZmZmZmZmZmQ$Jmw/A8cPvgLKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGKKPGK"),
    ];

    for (i, (username, email, password_hash)) in users.iter().enumerate() {
        let result = sqlx::query!(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES (?, ?, ?)
            "#,
            username,
            email,
            password_hash
        )
        .execute(pool)
        .await?;

        println!(
            "      User {} created with ID: {}",
            i + 1,
            result.last_insert_rowid()
        );
    }

    println!("   ✓ Created {} users", users.len());
    Ok(())
}

/// Seed categories table
async fn seed_categories(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("🏷️  Seeding categories...");

    // Categories for user 1 (alice_wang)
    let user1_categories = vec![
        // Income categories
        "Salary",
        "Bonus",
        "Freelance",
        "Investment Returns",
        "Gift Received",
        // Expense categories
        "Groceries",
        "Dining Out",
        "Transportation",
        "Gas",
        "Public Transit",
        "Rent",
        "Utilities",
        "Electricity",
        "Water",
        "Internet",
        "Phone",
        "Entertainment",
        "Movies",
        "Concerts",
        "Shopping",
        "Clothing",
        "Electronics",
        "Healthcare",
        "Insurance",
        "Fitness",
        "Education",
        "Travel",
        "Subscriptions",
    ];

    // Categories for user 2 (bob_chen)
    let user2_categories = vec![
        "Salary",
        "Food",
        "Transportation",
        "Housing",
        "Entertainment",
        "Shopping",
        "Healthcare",
        "Savings",
    ];

    // Categories for user 3 (carol_liu)
    let user3_categories = vec![
        "Income",
        "Groceries",
        "Restaurants",
        "Car",
        "Rent",
        "Fun",
        "Misc",
    ];

    let mut total = 0;

    // Insert user 1 categories
    for name in user1_categories.iter() {
        sqlx::query!(
            r#"INSERT INTO categories (user_id, name) VALUES (1, ?)"#,
            name
        )
        .execute(pool)
        .await?;
        total += 1;
    }

    // Insert user 2 categories
    for name in user2_categories.iter() {
        sqlx::query!(
            r#"INSERT INTO categories (user_id, name) VALUES (2, ?)"#,
            name
        )
        .execute(pool)
        .await?;
        total += 1;
    }

    // Insert user 3 categories
    for name in user3_categories.iter() {
        sqlx::query!(
            r#"INSERT INTO categories (user_id, name) VALUES (3, ?)"#,
            name
        )
        .execute(pool)
        .await?;
        total += 1;
    }

    println!("   ✓ Created {} categories", total);
    Ok(())
}

/// Seed accounts table
async fn seed_accounts(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("💳 Seeding accounts...");

    let accounts = vec![
        // User 1 (alice_wang) accounts
        (
            1,
            "Chase Checking",
            "checking",
            Some("Chase Bank"),
            "USD",
            5000.0,
            5000.0,
        ),
        (
            1,
            "Ally Savings",
            "savings",
            Some("Ally Bank"),
            "USD",
            15000.0,
            15000.0,
        ),
        (
            1,
            "Chase Sapphire Card",
            "credit_card",
            Some("Chase"),
            "USD",
            0.0,
            -850.0,
        ),
        (
            1,
            "EUR Travel Account",
            "checking",
            Some("Wise"),
            "EUR",
            1000.0,
            1000.0,
        ),
        // User 2 (bob_chen) accounts
        (
            2,
            "Main Checking",
            "checking",
            Some("Bank of America"),
            "USD",
            3000.0,
            3000.0,
        ),
        (
            2,
            "Emergency Fund",
            "savings",
            Some("Marcus"),
            "USD",
            10000.0,
            10000.0,
        ),
        (
            2,
            "Credit Card",
            "credit_card",
            Some("Capital One"),
            "USD",
            0.0,
            -500.0,
        ),
        // User 3 (carol_liu) accounts
        (
            3,
            "Checking",
            "checking",
            Some("Wells Fargo"),
            "USD",
            2500.0,
            2500.0,
        ),
        (
            3,
            "Savings",
            "savings",
            Some("Wells Fargo"),
            "USD",
            8000.0,
            8000.0,
        ),
    ];

    for (user_id, name, account_type, bank_name, currency, initial, current) in accounts.iter() {
        sqlx::query!(
            r#"
            INSERT INTO accounts 
            (user_id, name, account_type, bank_name, currency, initial_balance, current_balance)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            user_id,
            name,
            account_type,
            bank_name,
            currency,
            initial,
            current
        )
        .execute(pool)
        .await?;
    }

    println!("   ✓ Created {} accounts", accounts.len());
    Ok(())
}

/// Seed transactions table
async fn seed_transactions(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("💰 Seeding transactions...");

    let now = Utc::now();

    // User 1 (alice_wang) transactions
    let user1_transactions = vec![
        // This month
        (1, 5000.0, "income", "Monthly salary", 0),
        (1, -1500.0, "expense", "Monthly rent", 0),
        (1, -150.0, "expense", "Grocery shopping at Whole Foods", -2),
        (1, -80.0, "expense", "Dinner with friends", -3),
        (1, -50.0, "expense", "Gas station", -4),
        (1, -120.0, "expense", "Electric bill", -5),
        (1, -60.0, "expense", "Water bill", -5),
        (1, -100.0, "expense", "Internet bill", -1),
        (1, -45.0, "expense", "Phone bill", -1),
        (1, -200.0, "expense", "Shopping at Target", -7),
        (1, -30.0, "expense", "Netflix subscription", -10),
        // Last month
        (1, 5000.0, "income", "Monthly salary", -30),
        (1, -1500.0, "expense", "Monthly rent", -30),
        (1, -200.0, "expense", "Groceries", -32),
        (1, -100.0, "expense", "Restaurants", -35),
        (1, 500.0, "income", "Freelance project", -20),
        // Credit card transactions
        (3, -350.0, "expense", "Amazon purchases", -5),
        (3, -250.0, "expense", "Flight tickets", -15),
        (3, -250.0, "expense", "Hotel booking", -15),
        // Savings account
        (2, 1000.0, "income", "Transfer from checking", -1),
    ];

    // User 2 (bob_chen) transactions
    let user2_transactions = vec![
        (5, 4000.0, "income", "Salary", 0),
        (5, -1200.0, "expense", "Rent", 0),
        (5, -100.0, "expense", "Groceries", -3),
        (5, -50.0, "expense", "Gas", -5),
        (5, -80.0, "expense", "Restaurants", -7),
        (7, -200.0, "expense", "Online shopping", -10),
    ];

    // User 3 (carol_liu) transactions
    let user3_transactions = vec![
        (8, 3500.0, "income", "Paycheck", 0),
        (8, -1000.0, "expense", "Rent", 0),
        (8, -80.0, "expense", "Groceries", -2),
        (8, -40.0, "expense", "Coffee shop", -5),
    ];

    let mut total = 0;

    // Insert user 1 transactions
    for (account_id, amount, txn_type, desc, days_offset) in user1_transactions.iter() {
        let txn_date = now + Duration::days(*days_offset);
        sqlx::query!(
            r#"
            INSERT INTO transactions 
            (account_id, amount, transaction_type, description, transaction_date)
            VALUES (?, ?, ?, ?, ?)
            "#,
            account_id,
            amount,
            txn_type,
            desc,
            txn_date
        )
        .execute(pool)
        .await?;
        total += 1;
    }

    // Insert user 2 transactions
    for (account_id, amount, txn_type, desc, days_offset) in user2_transactions.iter() {
        let txn_date = now + Duration::days(*days_offset);
        sqlx::query!(
            r#"
            INSERT INTO transactions 
            (account_id, amount, transaction_type, description, transaction_date)
            VALUES (?, ?, ?, ?, ?)
            "#,
            account_id,
            amount,
            txn_type,
            desc,
            txn_date
        )
        .execute(pool)
        .await?;
        total += 1;
    }

    // Insert user 3 transactions
    for (account_id, amount, txn_type, desc, days_offset) in user3_transactions.iter() {
        let txn_date = now + Duration::days(*days_offset);
        sqlx::query!(
            r#"
            INSERT INTO transactions 
            (account_id, amount, transaction_type, description, transaction_date)
            VALUES (?, ?, ?, ?, ?)
            "#,
            account_id,
            amount,
            txn_type,
            desc,
            txn_date
        )
        .execute(pool)
        .await?;
        total += 1;
    }

    println!("   ✓ Created {} transactions", total);
    Ok(())
}

/// Seed transaction_categories table (linking transactions to categories)
async fn seed_transaction_categories(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("🔗 Seeding transaction categories...");

    // Category ID reference (based on insertion order):
    // User 1 (alice_wang): IDs 1-28
    //   1: Salary, 2: Bonus, 3: Freelance, 4: Investment Returns, 5: Gift Received
    //   6: Groceries, 7: Dining Out, 8: Transportation, 9: Gas, 10: Public Transit
    //   11: Rent, 12: Utilities, 13: Electricity, 14: Water, 15: Internet
    //   16: Phone, 17: Entertainment, 18: Movies, 19: Concerts, 20: Shopping
    //   21: Clothing, 22: Electronics, 23: Healthcare, 24: Insurance, 25: Fitness
    //   26: Education, 27: Travel, 28: Subscriptions
    // User 2 (bob_chen): IDs 29-36
    //   29: Salary, 30: Food, 31: Transportation, 32: Housing, 33: Entertainment
    //   34: Shopping, 35: Healthcare, 36: Savings
    // User 3 (carol_liu): IDs 37-43
    //   37: Income, 38: Groceries, 39: Restaurants, 40: Car, 41: Rent
    //   42: Fun, 43: Misc

    // Simple 1:1 mappings (transaction_id -> category_id, amount)
    let mappings = vec![
        // User 1 income
        (1, 1, 5000.0),  // Salary
        (12, 1, 5000.0), // Salary (last month)
        (16, 3, 500.0),  // Freelance
        (20, 1, 1000.0), // Transfer (counted as savings/income)
        // User 1 expenses
        (2, 11, 1500.0),  // Rent
        (3, 6, 150.0),    // Groceries
        (4, 7, 80.0),     // Dining out
        (5, 9, 50.0),     // Gas
        (6, 13, 120.0),   // Electricity
        (7, 14, 60.0),    // Water
        (8, 15, 100.0),   // Internet
        (9, 16, 45.0),    // Phone
        (10, 20, 200.0),  // Shopping
        (11, 28, 30.0),   // Subscriptions (Netflix)
        (13, 11, 1500.0), // Rent (last month)
        (14, 6, 200.0),   // Groceries (last month)
        (15, 7, 100.0),   // Restaurants (last month)
        // Credit card
        (17, 22, 350.0), // Electronics/Shopping
        (18, 27, 250.0), // Travel
        (19, 27, 250.0), // Travel
        // User 2
        (21, 29, 4000.0), // Salary
        (22, 32, 1200.0), // Housing
        (23, 30, 100.0),  // Food
        (24, 31, 50.0),   // Transportation
        (25, 33, 80.0),   // Entertainment
        (26, 34, 200.0),  // Shopping
        // User 3
        (27, 37, 3500.0), // Income
        (28, 41, 1000.0), // Rent
        (29, 38, 80.0),   // Groceries
        (30, 39, 40.0),   // Restaurants
    ];

    for (transaction_id, category_id, amount) in mappings.iter() {
        sqlx::query!(
            r#"
            INSERT INTO transaction_categories (transaction_id, category_id, amount)
            VALUES (?, ?, ?)
            "#,
            transaction_id,
            category_id,
            amount
        )
        .execute(pool)
        .await?;
    }

    println!("   ✓ Created {} transaction-category links", mappings.len());
    Ok(())
}

/// Seed recurring_transactions table
async fn seed_recurring_transactions(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("🔄 Seeding recurring transactions...");

    let now = Utc::now();
    let next_month = now + Duration::days(30);

    // Define None with explicit type for end_date
    let no_end_date: Option<DateTime<Utc>> = None;

    let recurring = vec![
        // User 1 recurring transactions
        (
            1,
            Some(1),
            5000.0,
            "income",
            Some("Monthly salary"),
            "monthly",
            now,
            no_end_date,
            next_month,
            true,
        ),
        (
            1,
            Some(11),
            -1500.0,
            "expense",
            Some("Monthly rent"),
            "monthly",
            now,
            no_end_date,
            next_month,
            true,
        ),
        (
            1,
            Some(15),
            -100.0,
            "expense",
            Some("Internet bill"),
            "monthly",
            now,
            no_end_date,
            next_month,
            true,
        ),
        (
            1,
            Some(16),
            -45.0,
            "expense",
            Some("Phone bill"),
            "monthly",
            now,
            no_end_date,
            next_month,
            true,
        ),
        (
            1,
            Some(28),
            -30.0,
            "expense",
            Some("Netflix subscription"),
            "monthly",
            now,
            no_end_date,
            next_month,
            true,
        ),
        // User 2 recurring transactions
        (
            5,
            Some(29),
            4000.0,
            "income",
            Some("Monthly salary"),
            "monthly",
            now,
            no_end_date,
            next_month,
            true,
        ),
        (
            5,
            Some(32),
            -1200.0,
            "expense",
            Some("Rent payment"),
            "monthly",
            now,
            no_end_date,
            next_month,
            true,
        ),
        // User 3 recurring transactions
        (
            8,
            Some(43),
            3500.0,
            "income",
            Some("Salary"),
            "monthly",
            now,
            no_end_date,
            next_month,
            true,
        ),
        (
            8,
            Some(41),
            -1000.0,
            "expense",
            Some("Rent"),
            "monthly",
            now,
            no_end_date,
            next_month,
            true,
        ),
    ];

    for (account_id, category_id, amount, txn_type, desc, freq, start, end, next, active) in
        recurring.iter()
    {
        sqlx::query!(
            r#"
            INSERT INTO recurring_transactions 
            (account_id, category_id, amount, transaction_type, description, 
             frequency, start_date, end_date, next_occurrence, is_active)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            account_id,
            category_id,
            amount,
            txn_type,
            desc,
            freq,
            start,
            end,
            next,
            active
        )
        .execute(pool)
        .await?;
    }

    println!("   ✓ Created {} recurring transactions", recurring.len());
    Ok(())
}

/// Seed exchange_rates table
async fn seed_exchange_rates(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("💱 Seeding exchange rates...");

    let now = Utc::now();
    let yesterday = now - Duration::days(1);

    let rates = vec![
        // Today's rates
        ("USD", "EUR", 0.92, now, "api"),
        ("USD", "GBP", 0.79, now, "api"),
        ("USD", "JPY", 149.50, now, "api"),
        ("USD", "CNY", 7.24, now, "api"),
        ("USD", "CAD", 1.36, now, "api"),
        ("EUR", "USD", 1.09, now, "api"),
        ("EUR", "GBP", 0.86, now, "api"),
        ("GBP", "USD", 1.27, now, "api"),
        // Yesterday's rates (for historical comparison)
        ("USD", "EUR", 0.93, yesterday, "api"),
        ("USD", "GBP", 0.80, yesterday, "api"),
        ("USD", "JPY", 148.80, yesterday, "api"),
    ];

    for (from, to, rate, date, source) in rates.iter() {
        sqlx::query!(
            r#"
            INSERT INTO exchange_rates 
            (from_currency, to_currency, rate, rate_date, source)
            VALUES (?, ?, ?, ?, ?)
            "#,
            from,
            to,
            rate,
            date,
            source
        )
        .execute(pool)
        .await?;
    }

    println!("   ✓ Created {} exchange rates", rates.len());
    Ok(())
}

/// Print a summary of seeded data
async fn print_seed_summary(pool: &SqlitePool) -> Result<(), sqlx::Error> {
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

    println!("📊 Seed Summary:");
    println!("   • {} users", users);
    println!("   • {} accounts", accounts);
    println!("   • {} categories", categories);
    println!("   • {} transactions", transactions);
    println!("   • {} recurring transactions", recurring);
    println!("   • {} exchange rates", rates);

    Ok(())
}

/// Clear all data from the database (useful for re-seeding)
pub async fn clear_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("🗑️  Clearing database...");

    // Delete in reverse order of foreign key dependencies
    sqlx::query!("DELETE FROM transaction_categories")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM transactions")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM recurring_transactions")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM exchange_rates")
        .execute(pool)
        .await?;
    sqlx::query!("DELETE FROM accounts").execute(pool).await?;
    sqlx::query!("DELETE FROM categories").execute(pool).await?;
    sqlx::query!("DELETE FROM users").execute(pool).await?;

    println!("   ✓ All data cleared");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_seed_database() {
        // This test requires a test database
        // You can set up a test database in your test configuration
    }
}
