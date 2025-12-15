use chrono::{Datelike, Duration, TimeZone, Utc};
use sqlx::SqlitePool;

use crate::models::RecurringTransaction;

/// Result from processing recurring transactions.
pub struct RecurringProcessResult {
    pub due: usize,
    pub created: usize,
}

/// Process all due recurring transactions:
/// - create concrete transactions
/// - link categories
/// - update account balances
/// - advance next_occurrence or deactivate when past end_date
pub async fn process_due_recurring(
    pool: &SqlitePool,
) -> Result<RecurringProcessResult, sqlx::Error> {
    let now = Utc::now();

    let transactions = sqlx::query_as::<_, RecurringTransaction>(
        "SELECT * FROM recurring_transactions 
         WHERE is_active = 1 AND next_occurrence <= ? 
         AND (end_date IS NULL OR end_date > ?)",
    )
    .bind(now)
    .bind(now)
    .fetch_all(pool)
    .await?;

    let mut created_count = 0;

    for recurring in &transactions {
        let result = sqlx::query(
            "INSERT INTO transactions (account_id, amount, transaction_type, description, transaction_date) 
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(recurring.account_id)
        .bind(recurring.amount)
        .bind(&recurring.transaction_type)
        .bind(&recurring.description)
        .bind(recurring.next_occurrence)
        .execute(pool)
        .await;

        if let Ok(res) = result {
            let transaction_id = res.last_insert_rowid();

            // Link category if exists
            if let Some(category_id) = recurring.category_id {
                let _ = sqlx::query(
                    "INSERT INTO transaction_categories (transaction_id, category_id, amount) 
                     VALUES (?, ?, ?)",
                )
                .bind(transaction_id)
                .bind(category_id)
                .bind(recurring.amount.abs())
                .execute(pool)
                .await;
            }

            // Update account balance
            let balance_change = if recurring.transaction_type == "income" {
                recurring.amount
            } else {
                -recurring.amount.abs()
            };

            let _ = sqlx::query(
                "UPDATE accounts SET current_balance = current_balance + ? WHERE id = ?",
            )
            .bind(balance_change)
            .bind(recurring.account_id)
            .execute(pool)
            .await;

            // Calculate next occurrence
            let next = calculate_next_occurrence(recurring.next_occurrence, &recurring.frequency);

            // Check if should deactivate (past end_date)
            let should_deactivate = recurring
                .end_date
                .map(|end| next > end)
                .unwrap_or(false);

            if should_deactivate {
                let _ = sqlx::query(
                    "UPDATE recurring_transactions SET is_active = 0, next_occurrence = ?, updated_at = datetime('now') WHERE id = ?",
                )
                .bind(next)
                .bind(recurring.id)
                .execute(pool)
                .await;
            } else {
                let _ = sqlx::query(
                    "UPDATE recurring_transactions SET next_occurrence = ?, updated_at = datetime('now') WHERE id = ?",
                )
                .bind(next)
                .bind(recurring.id)
                .execute(pool)
                .await;
            }

            created_count += 1;
        }
    }

    Ok(RecurringProcessResult {
        due: transactions.len(),
        created: created_count,
    })
}

fn calculate_next_occurrence(current: chrono::DateTime<Utc>, frequency: &str) -> chrono::DateTime<Utc> {
    match frequency {
        "daily" => current + Duration::days(1),
        "weekly" => current + Duration::weeks(1),
        "monthly" => {
            let year = current.year();
            let month = current.month();
            let day = current.day();

            let (new_year, new_month) = if month == 12 {
                (year + 1, 1)
            } else {
                (year, month + 1)
            };

            let days_in_new_month = days_in_month(new_year, new_month);
            let new_day = day.min(days_in_new_month);
            

            Utc.with_ymd_and_hms(new_year, new_month, new_day, 0, 0, 0)
                .unwrap()
        }
        "yearly" => {
            let year = current.year();
            let month = current.month();
            let day = current.day();

            let new_day = if month == 2 && day == 29 {
                if is_leap_year(year + 1) { 29 } else { 28 }
            } else {
                day
            };

            Utc.with_ymd_and_hms(year + 1, month, new_day, 0, 0, 0)
                .unwrap()
        }
        _ => current + Duration::days(30), // Default to monthly
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
