use crate::models::*;
use crate::recurring;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::SqlitePool;

// ============================================================================
// User Endpoints
// ============================================================================

/// GET /users - List all users (paginated)
#[get("/users")]
async fn get_users(
    pool: web::Data<SqlitePool>,
    query: web::Query<PaginationParams>,
) -> impl Responder {
    let offset = (query.page - 1) * query.page_size;

    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(query.page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    match users {
        Ok(users) => {
            let response = PaginatedResponse {
                items: users,
                total,
                page: query.page,
                page_size: query.page_size,
                total_pages: (total + query.page_size - 1) / query.page_size,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /users/{id} - Get user by ID
#[get("/users/{id}")]
async fn get_user(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let id = id.into_inner();

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse::success(user)),
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found".into()))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// POST /users - Create new user
#[post("/users")]
async fn create_user(
    pool: web::Data<SqlitePool>,
    user_data: web::Json<CreateUser>,
) -> impl Responder {
    if let Err(e) = user_data.validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(e));
    }

    let password_hash = format!("$argon2id$v=19$m=19456,t=2,p=1${}", user_data.password);

    let result = sqlx::query("INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)")
        .bind(&user_data.username)
        .bind(&user_data.email)
        .bind(&password_hash)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            let user = sqlx::query_as::<_, User>(
                "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE id = ?"
            )
            .bind(result.last_insert_rowid())
            .fetch_one(pool.get_ref())
            .await
            .unwrap();

            HttpResponse::Created().json(ApiResponse::success(user))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// PUT /users/{id} - Update user
#[put("/users/{id}")]
async fn update_user(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
    update_data: web::Json<UpdateUser>,
) -> impl Responder {
    let id = id.into_inner();

    let mut updates = Vec::new();
    let mut query = String::from("UPDATE users SET ");

    if let Some(username) = &update_data.username {
        updates.push(format!("username = '{}'", username));
    }
    if let Some(email) = &update_data.email {
        updates.push(format!("email = '{}'", email));
    }
    if let Some(password) = &update_data.password {
        let hash = format!("$argon2id$v=19$m=19456,t=2,p=1${}", password);
        updates.push(format!("password_hash = '{}'", hash));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("No fields to update".into()));
    }

    query.push_str(&updates.join(", "));
    query.push_str(&format!(", updated_at = datetime('now') WHERE id = {}", id));

    let result = sqlx::query(&query).execute(pool.get_ref()).await;

    match result {
        Ok(_) => {
            let user = sqlx::query_as::<_, User>(
                "SELECT id, username, email, password_hash, created_at, updated_at FROM users WHERE id = ?"
            )
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .unwrap();
            HttpResponse::Ok().json(ApiResponse::success(user))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// DELETE /users/{id} - Delete user
#[delete("/users/{id}")]
async fn delete_user(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let id = id.into_inner();

    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::success("User deleted successfully"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("User not found".into()))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

// ============================================================================
// Account Endpoints
// ============================================================================

/// GET /accounts - List all accounts
#[get("/accounts")]
async fn get_accounts(
    pool: web::Data<SqlitePool>,
    query: web::Query<PaginationParams>,
) -> impl Responder {
    let offset = (query.page - 1) * query.page_size;

    let accounts = sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(query.page_size)
    .bind(offset)
    .fetch_all(pool.get_ref())
    .await;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    match accounts {
        Ok(accounts) => {
            let response = PaginatedResponse {
                items: accounts,
                total,
                page: query.page,
                page_size: query.page_size,
                total_pages: (total + query.page_size - 1) / query.page_size,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /accounts/{id} - Get account by ID
#[get("/accounts/{id}")]
async fn get_account(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let id = id.into_inner();

    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;

    match account {
        Ok(Some(account)) => HttpResponse::Ok().json(ApiResponse::success(account)),
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<()>::error("Account not found".into()))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// POST /accounts - Create new account
#[post("/accounts")]
async fn create_account(
    pool: web::Data<SqlitePool>,
    account_data: web::Json<CreateAccount>,
) -> impl Responder {
    if let Err(e) = account_data.validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(e));
    }

    let currency = account_data.currency.as_deref().unwrap_or("USD");
    let initial_balance = account_data.initial_balance.unwrap_or(0.0);

    let result = sqlx::query(
        "INSERT INTO accounts (user_id, name, account_type, bank_name, currency, initial_balance, current_balance) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(account_data.user_id)
    .bind(&account_data.name)
    .bind(&account_data.account_type)
    .bind(&account_data.bank_name)
    .bind(currency)
    .bind(initial_balance)
    .bind(initial_balance)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
                .bind(result.last_insert_rowid())
                .fetch_one(pool.get_ref())
                .await
                .unwrap();

            HttpResponse::Created().json(ApiResponse::success(account))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// PUT /accounts/{id} - Update account
#[put("/accounts/{id}")]
async fn update_account(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
    update_data: web::Json<UpdateAccount>,
) -> impl Responder {
    let id = id.into_inner();
    let mut updates = Vec::new();

    if let Some(name) = &update_data.name {
        updates.push(format!("name = '{}'", name));
    }
    if let Some(account_type) = &update_data.account_type {
        updates.push(format!("account_type = '{}'", account_type));
    }
    if let Some(bank_name) = &update_data.bank_name {
        updates.push(format!("bank_name = '{}'", bank_name));
    }
    if let Some(currency) = &update_data.currency {
        updates.push(format!("currency = '{}'", currency));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("No fields to update".into()));
    }

    let query = format!(
        "UPDATE accounts SET {}, updated_at = datetime('now') WHERE id = {}",
        updates.join(", "),
        id
    );

    let result = sqlx::query(&query).execute(pool.get_ref()).await;

    match result {
        Ok(_) => {
            let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
                .bind(id)
                .fetch_one(pool.get_ref())
                .await
                .unwrap();
            HttpResponse::Ok().json(ApiResponse::success(account))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// DELETE /accounts/{id} - Delete account
#[delete("/accounts/{id}")]
async fn delete_account(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let id = id.into_inner();

    let result = sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::success("Account deleted successfully"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Account not found".into()))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

// ============================================================================
// Category Endpoints
// ============================================================================

/// GET /categories - List all categories
#[get("/categories")]
async fn get_categories(
    pool: web::Data<SqlitePool>,
    query: web::Query<PaginationParams>,
) -> impl Responder {
    let offset = (query.page - 1) * query.page_size;

    let categories =
        sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY name LIMIT ? OFFSET ?")
            .bind(query.page_size)
            .bind(offset)
            .fetch_all(pool.get_ref())
            .await;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories")
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    match categories {
        Ok(categories) => {
            let response = PaginatedResponse {
                items: categories,
                total,
                page: query.page,
                page_size: query.page_size,
                total_pages: (total + query.page_size - 1) / query.page_size,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /categories/{id} - Get category by ID
#[get("/categories/{id}")]
async fn get_category(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let id = id.into_inner();

    let category = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;

    match category {
        Ok(Some(category)) => HttpResponse::Ok().json(ApiResponse::success(category)),
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<()>::error("Category not found".into()))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// POST /categories - Create new category
#[post("/categories")]
async fn create_category(
    pool: web::Data<SqlitePool>,
    category_data: web::Json<CreateCategory>,
) -> impl Responder {
    let result = sqlx::query("INSERT INTO categories (user_id, name) VALUES (?, ?)")
        .bind(category_data.user_id)
        .bind(&category_data.name)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            let category = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?")
                .bind(result.last_insert_rowid())
                .fetch_one(pool.get_ref())
                .await
                .unwrap();

            HttpResponse::Created().json(ApiResponse::success(category))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// PUT /categories/{id} - Update category
#[put("/categories/{id}")]
async fn update_category(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
    update_data: web::Json<UpdateCategory>,
) -> impl Responder {
    let id = id.into_inner();

    if let Some(name) = &update_data.name {
        let result = sqlx::query(
            "UPDATE categories SET name = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(name)
        .bind(id)
        .execute(pool.get_ref())
        .await;

        match result {
            Ok(_) => {
                let category =
                    sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?")
                        .bind(id)
                        .fetch_one(pool.get_ref())
                        .await
                        .unwrap();
                HttpResponse::Ok().json(ApiResponse::success(category))
            }
            Err(e) => {
                HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string()))
            }
        }
    } else {
        HttpResponse::BadRequest().json(ApiResponse::<()>::error("No name provided".into()))
    }
}

/// DELETE /categories/{id} - Delete category
#[delete("/categories/{id}")]
async fn delete_category(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let id = id.into_inner();

    let result = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::success("Category deleted successfully"))
            } else {
                HttpResponse::NotFound().json(ApiResponse::<()>::error("Category not found".into()))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

// ============================================================================
// Transaction Endpoints
// ============================================================================

/// GET /transactions - List transactions with filters
#[get("/transactions")]
async fn get_transactions(
    pool: web::Data<SqlitePool>,
    query: web::Query<TransactionFilter>,
) -> impl Responder {
    let offset = (query.page - 1) * query.page_size;

    let mut where_clauses = Vec::new();

    if let Some(account_id) = query.account_id {
        where_clauses.push(format!("account_id = {}", account_id));
    }
    if let Some(ref txn_type) = query.transaction_type {
        where_clauses.push(format!("transaction_type = '{}'", txn_type));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let query_sql = format!(
        "SELECT * FROM transactions {} ORDER BY transaction_date DESC LIMIT {} OFFSET {}",
        where_sql, query.page_size, offset
    );

    let transactions = sqlx::query_as::<_, Transaction>(&query_sql)
        .fetch_all(pool.get_ref())
        .await;

    let count_sql = format!("SELECT COUNT(*) FROM transactions {}", where_sql);
    let total: i64 = sqlx::query_scalar(&count_sql)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    match transactions {
        Ok(transactions) => {
            let response = PaginatedResponse {
                items: transactions,
                total,
                page: query.page,
                page_size: query.page_size,
                total_pages: (total + query.page_size - 1) / query.page_size,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /transactions/{id} - Get transaction by ID with categories
#[get("/transactions/{id}")]
async fn get_transaction(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    use sqlx::Row; // Add this import at the top of the function

    let id = id.into_inner();

    let transaction = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;

    match transaction {
        Ok(Some(transaction)) => {
            // Manually fetch category data
            let category_rows = sqlx::query(
                "SELECT tc.category_id, c.name as category_name, tc.amount 
                 FROM transaction_categories tc 
                 JOIN categories c ON tc.category_id = c.id 
                 WHERE tc.transaction_id = ?",
            )
            .bind(id)
            .fetch_all(pool.get_ref())
            .await
            .unwrap_or_default();

            // Manually construct TransactionCategoryDetail
            let categories: Vec<TransactionCategoryDetail> = category_rows
                .iter()
                .filter_map(|row| {
                    Some(TransactionCategoryDetail {
                        category_id: row.try_get("category_id").ok()?,
                        category_name: row.try_get("category_name").ok()?,
                        amount: row.try_get("amount").ok()?,
                    })
                })
                .collect();

            let response = TransactionWithCategories {
                transaction,
                categories,
            };

            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<()>::error("Transaction not found".into()))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// POST /transactions - Create new transaction
#[post("/transactions")]
async fn create_transaction(
    pool: web::Data<SqlitePool>,
    txn_data: web::Json<CreateTransaction>,
) -> impl Responder {
    if let Err(e) = txn_data.validate() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(e));
    }

    let txn_date = txn_data.transaction_date.unwrap_or_else(Utc::now);

    let result = sqlx::query(
        "INSERT INTO transactions (account_id, amount, transaction_type, description, transaction_date) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(txn_data.account_id)
    .bind(txn_data.amount)
    .bind(&txn_data.transaction_type)
    .bind(&txn_data.description)
    .bind(txn_date)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            let transaction_id = result.last_insert_rowid();

            for cat_amount in &txn_data.categories {
                let _ = sqlx::query(
                    "INSERT INTO transaction_categories (transaction_id, category_id, amount) VALUES (?, ?, ?)"
                )
                .bind(transaction_id)
                .bind(cat_amount.category_id)
                .bind(cat_amount.amount)
                .execute(pool.get_ref())
                .await;
            }

            let balance_change = if txn_data.transaction_type == "income" {
                txn_data.amount
            } else {
                -txn_data.amount.abs()
            };

            let _ = sqlx::query(
                "UPDATE accounts SET current_balance = current_balance + ? WHERE id = ?",
            )
            .bind(balance_change)
            .bind(txn_data.account_id)
            .execute(pool.get_ref())
            .await;

            let transaction =
                sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?")
                    .bind(transaction_id)
                    .fetch_one(pool.get_ref())
                    .await
                    .unwrap();

            HttpResponse::Created().json(ApiResponse::success(transaction))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// PUT /transactions/{id} - Update transaction
#[put("/transactions/{id}")]
async fn update_transaction(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
    update_data: web::Json<UpdateTransaction>,
) -> impl Responder {
    let id = id.into_inner();
    let mut updates = Vec::new();

    if let Some(amount) = update_data.amount {
        updates.push(format!("amount = {}", amount));
    }
    if let Some(ref txn_type) = update_data.transaction_type {
        updates.push(format!("transaction_type = '{}'", txn_type));
    }
    if let Some(ref desc) = update_data.description {
        updates.push(format!("description = '{}'", desc));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("No fields to update".into()));
    }

    let query = format!(
        "UPDATE transactions SET {}, updated_at = datetime('now') WHERE id = {}",
        updates.join(", "),
        id
    );

    let result = sqlx::query(&query).execute(pool.get_ref()).await;

    match result {
        Ok(_) => {
            let transaction =
                sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?")
                    .bind(id)
                    .fetch_one(pool.get_ref())
                    .await
                    .unwrap();
            HttpResponse::Ok().json(ApiResponse::success(transaction))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// DELETE /transactions/{id} - Delete transaction
#[delete("/transactions/{id}")]
async fn delete_transaction(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let id = id.into_inner();

    // 1. Fetch the transaction so we know its amount, type, and account
    let existing_txn = sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;

    let txn = match existing_txn {
        Ok(Some(txn)) => txn,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<()>::error("Transaction not found".into()))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(e.to_string()))
        }
    };

    // 2. Compute the reverse balance change
    let balance_change = if txn.transaction_type == "income" {
        // Creation: +amount  → Deletion: -amount
        -txn.amount
    } else {
        // Creation: -amount.abs() → Deletion: +amount.abs()
        txn.amount.abs()
    };

    // 3. Delete any related transaction_categories rows (if you have them)
    if let Err(e) = sqlx::query("DELETE FROM transaction_categories WHERE transaction_id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await
    {
        return HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string()));
    }

    // 4. Delete the transaction itself
    let result = sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // 5. Apply the balance update to the account
                let _ = sqlx::query(
                    "UPDATE accounts SET current_balance = current_balance + ? WHERE id = ?",
                )
                .bind(balance_change)
                .bind(txn.account_id)
                .execute(pool.get_ref())
                .await;

                HttpResponse::Ok().json(ApiResponse::success("Transaction deleted successfully"))
            } else {
                // Shouldn’t really happen since we already fetched it,
                // but keep the check for safety.
                HttpResponse::NotFound()
                    .json(ApiResponse::<()>::error("Transaction not found".into()))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

// ============================================================================
// Exchange Rate Endpoints
// ============================================================================

/// GET /exchange-rates - List exchange rates with filters
#[get("/exchange-rates")]
async fn get_exchange_rates(
    pool: web::Data<SqlitePool>,
    query: web::Query<ExchangeRateFilter>,
) -> impl Responder {
    let offset = (query.page - 1) * query.page_size as i64;

    let mut where_clauses = Vec::new();

    if let Some(ref from) = query.from_currency {
        where_clauses.push(format!("from_currency = '{}'", from));
    }
    if let Some(ref to) = query.to_currency {
        where_clauses.push(format!("to_currency LIKE '%{}%'", to));
    }
    if let Some(ref source) = query.source {
        where_clauses.push(format!("source = '{}'", source));
    }
    if let Some(date) = query.date {
        where_clauses.push(format!("DATE(rate_date) = '{}'", date.format("%Y-%m-%d")));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let query_sql = format!(
        "SELECT * FROM exchange_rates {} ORDER BY rate_date DESC, from_currency, to_currency LIMIT {} OFFSET {}",
        where_sql, query.page_size, offset
    );

    let rates = sqlx::query_as::<_, ExchangeRate>(&query_sql)
        .fetch_all(pool.get_ref())
        .await;

    let count_sql = format!("SELECT COUNT(*) FROM exchange_rates {}", where_sql);
    let total: i64 = sqlx::query_scalar(&count_sql)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    match rates {
        Ok(rates) => {
            let response = PaginatedResponse {
                items: rates,
                total,
                page: query.page,
                page_size: query.page_size,
                total_pages: (total + query.page_size - 1) / query.page_size,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /exchange-rates/latest/{from_currency} - Get latest rates for a currency
#[get("/exchange-rates/latest/{from_currency}")]
async fn get_latest_rates(
    pool: web::Data<SqlitePool>,
    from_currency: web::Path<String>,
) -> impl Responder {
    let from_currency = from_currency.into_inner();

    // Get the latest date for this currency
    let latest_date: Option<String> = sqlx::query_scalar(
        "SELECT DATE(rate_date) FROM exchange_rates 
         WHERE from_currency = ? 
         ORDER BY rate_date DESC 
         LIMIT 1",
    )
    .bind(&from_currency)
    .fetch_optional(pool.get_ref())
    .await
    .unwrap_or(None);

    if latest_date.is_none() {
        return HttpResponse::NotFound().json(ApiResponse::<()>::error(format!(
            "No rates found for {}",
            from_currency
        )));
    }

    let latest_date = latest_date.unwrap();

    // Get all rates for that date
    let rates = sqlx::query_as::<_, ExchangeRate>(
        "SELECT * FROM exchange_rates 
         WHERE from_currency = ? AND DATE(rate_date) = ?
         ORDER BY to_currency",
    )
    .bind(&from_currency)
    .bind(&latest_date)
    .fetch_all(pool.get_ref())
    .await;

    match rates {
        Ok(rates) => HttpResponse::Ok().json(ApiResponse::success(rates)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /exchange-rates/convert - Convert amount between currencies
#[get("/exchange-rates/convert")]
async fn convert_currency(
    pool: web::Data<SqlitePool>,
    query: web::Query<CurrencyConversion>,
) -> impl Responder {
    // Get the latest rate
    let rate: Option<f64> = sqlx::query_scalar(
        "SELECT rate FROM exchange_rates 
         WHERE from_currency = ? AND to_currency LIKE ?
         ORDER BY rate_date DESC 
         LIMIT 1",
    )
    .bind(&query.from_currency)
    .bind(format!("%({})%", &query.to_currency))
    .fetch_optional(pool.get_ref())
    .await
    .unwrap_or(None);

    match rate {
        Some(rate) => {
            let converted_amount = query.amount * rate;
            let result = ConversionResult {
                from_currency: query.from_currency.clone(),
                to_currency: query.to_currency.clone(),
                amount: query.amount,
                rate,
                converted_amount,
            };
            HttpResponse::Ok().json(ApiResponse::success(result))
        }
        None => HttpResponse::NotFound().json(ApiResponse::<()>::error(format!(
            "No exchange rate found from {} to {}",
            query.from_currency, query.to_currency
        ))),
    }
}

/// GET /exchange-rates/{id} - Get exchange rate by ID
#[get("/exchange-rates/{id}")]
async fn get_exchange_rate(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let id = id.into_inner();

    let rate = sqlx::query_as::<_, ExchangeRate>("SELECT * FROM exchange_rates WHERE id = ?")
        .bind(id)
        .fetch_optional(pool.get_ref())
        .await;

    match rate {
        Ok(Some(rate)) => HttpResponse::Ok().json(ApiResponse::success(rate)),
        Ok(None) => HttpResponse::NotFound()
            .json(ApiResponse::<()>::error("Exchange rate not found".into())),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// POST /exchange-rates - Create new exchange rate
#[post("/exchange-rates")]
async fn create_exchange_rate(
    pool: web::Data<SqlitePool>,
    rate_data: web::Json<CreateExchangeRate>,
) -> impl Responder {
    let rate_date = rate_data.rate_date.unwrap_or_else(Utc::now);
    let source = rate_data.source.as_deref().unwrap_or("manual");

    let result = sqlx::query(
        "INSERT INTO exchange_rates (from_currency, to_currency, rate, rate_date, source) 
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&rate_data.from_currency)
    .bind(&rate_data.to_currency)
    .bind(rate_data.rate)
    .bind(rate_date)
    .bind(source)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            let rate =
                sqlx::query_as::<_, ExchangeRate>("SELECT * FROM exchange_rates WHERE id = ?")
                    .bind(result.last_insert_rowid())
                    .fetch_one(pool.get_ref())
                    .await
                    .unwrap();

            HttpResponse::Created().json(ApiResponse::success(rate))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// PUT /exchange-rates/{id} - Update exchange rate
#[put("/exchange-rates/{id}")]
async fn update_exchange_rate(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
    update_data: web::Json<UpdateExchangeRate>,
) -> impl Responder {
    let id = id.into_inner();
    let mut updates = Vec::new();

    if let Some(rate) = update_data.rate {
        updates.push(format!("rate = {}", rate));
    }
    if let Some(ref source) = update_data.source {
        updates.push(format!("source = '{}'", source));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("No fields to update".into()));
    }

    let query = format!(
        "UPDATE exchange_rates SET {}, updated_at = datetime('now') WHERE id = {}",
        updates.join(", "),
        id
    );

    let result = sqlx::query(&query).execute(pool.get_ref()).await;

    match result {
        Ok(_) => {
            let rate =
                sqlx::query_as::<_, ExchangeRate>("SELECT * FROM exchange_rates WHERE id = ?")
                    .bind(id)
                    .fetch_one(pool.get_ref())
                    .await
                    .unwrap();
            HttpResponse::Ok().json(ApiResponse::success(rate))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// DELETE /exchange-rates/{id} - Delete exchange rate
#[delete("/exchange-rates/{id}")]
async fn delete_exchange_rate(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let id = id.into_inner();

    let result = sqlx::query("DELETE FROM exchange_rates WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(ApiResponse::success("Exchange rate deleted successfully"))
            } else {
                HttpResponse::NotFound()
                    .json(ApiResponse::<()>::error("Exchange rate not found".into()))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// DELETE /exchange-rates/bulk - Delete rates by date and source
#[delete("/exchange-rates/bulk")]
async fn delete_rates_bulk(
    pool: web::Data<SqlitePool>,
    query: web::Query<BulkDeleteParams>,
) -> impl Responder {
    let mut where_clauses = Vec::new();

    if let Some(ref from) = query.from_currency {
        where_clauses.push(format!("from_currency = '{}'", from));
    }
    if let Some(date) = query.date {
        where_clauses.push(format!("DATE(rate_date) = '{}'", date.format("%Y-%m-%d")));
    }
    if let Some(ref source) = query.source {
        where_clauses.push(format!("source = '{}'", source));
    }

    if where_clauses.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "No deletion criteria provided".into(),
        ));
    }

    let query_sql = format!(
        "DELETE FROM exchange_rates WHERE {}",
        where_clauses.join(" AND ")
    );

    let result = sqlx::query(&query_sql).execute(pool.get_ref()).await;

    match result {
        Ok(result) => HttpResponse::Ok().json(ApiResponse::success(format!(
            "Deleted {} exchange rate(s)",
            result.rows_affected()
        ))),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

// ============================================================================
// Recurring Transaction Endpoints
// ============================================================================

/// GET /recurring-transactions - List recurring transactions
#[get("/recurring-transactions")]
async fn get_recurring_transactions(
    pool: web::Data<SqlitePool>,
    query: web::Query<RecurringTransactionFilter>,
) -> impl Responder {
    let offset = (query.page - 1) * query.page_size;

    let mut where_clauses = Vec::new();

    if let Some(account_id) = query.account_id {
        where_clauses.push(format!("account_id = {}", account_id));
    }
    if let Some(is_active) = query.is_active {
        where_clauses.push(format!("is_active = {}", if is_active { 1 } else { 0 }));
    }
    if let Some(ref frequency) = query.frequency {
        where_clauses.push(format!("frequency = '{}'", frequency));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let query_sql = format!(
        "SELECT * FROM recurring_transactions {} ORDER BY next_occurrence ASC LIMIT {} OFFSET {}",
        where_sql, query.page_size, offset
    );

    let recurring = sqlx::query_as::<_, RecurringTransaction>(&query_sql)
        .fetch_all(pool.get_ref())
        .await;

    let count_sql = format!("SELECT COUNT(*) FROM recurring_transactions {}", where_sql);
    let total: i64 = sqlx::query_scalar(&count_sql)
        .fetch_one(pool.get_ref())
        .await
        .unwrap_or(0);

    match recurring {
        Ok(recurring) => {
            let response = PaginatedResponse {
                items: recurring,
                total,
                page: query.page,
                page_size: query.page_size,
                total_pages: (total + query.page_size - 1) / query.page_size,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /recurring-transactions/{id} - Get recurring transaction by ID
#[get("/recurring-transactions/{id}")]
async fn get_recurring_transaction(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> impl Responder {
    let id = id.into_inner();

    let recurring =
        sqlx::query_as::<_, RecurringTransaction>("SELECT * FROM recurring_transactions WHERE id = ?")
            .bind(id)
            .fetch_optional(pool.get_ref())
            .await;

    match recurring {
        Ok(Some(recurring)) => HttpResponse::Ok().json(ApiResponse::success(recurring)),
        Ok(None) => HttpResponse::NotFound()
            .json(ApiResponse::<()>::error("Recurring transaction not found".into())),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// POST /recurring-transactions - Create new recurring transaction
#[post("/recurring-transactions")]
async fn create_recurring_transaction(
    pool: web::Data<SqlitePool>,
    data: web::Json<CreateRecurringTransaction>,
) -> impl Responder {
    let next_occurrence = data.start_date;

    let result = sqlx::query(
        "INSERT INTO recurring_transactions 
         (account_id, category_id, amount, transaction_type, description, frequency, start_date, end_date, next_occurrence, is_active) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1)",
    )
    .bind(data.account_id)
    .bind(data.category_id)
    .bind(data.amount)
    .bind(&data.transaction_type)
    .bind(&data.description)
    .bind(&data.frequency)
    .bind(data.start_date)
    .bind(data.end_date)
    .bind(next_occurrence)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) => {
            let recurring = sqlx::query_as::<_, RecurringTransaction>(
                "SELECT * FROM recurring_transactions WHERE id = ?",
            )
            .bind(result.last_insert_rowid())
            .fetch_one(pool.get_ref())
            .await
            .unwrap();

            HttpResponse::Created().json(ApiResponse::success(recurring))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// PUT /recurring-transactions/{id} - Update recurring transaction
#[put("/recurring-transactions/{id}")]
async fn update_recurring_transaction(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
    update_data: web::Json<UpdateRecurringTransaction>,
) -> impl Responder {
    let id = id.into_inner();
    let mut updates = Vec::new();

    if let Some(category_id) = update_data.category_id {
        updates.push(format!("category_id = {}", category_id));
    }
    if let Some(amount) = update_data.amount {
        updates.push(format!("amount = {}", amount));
    }
    if let Some(ref txn_type) = update_data.transaction_type {
        updates.push(format!("transaction_type = '{}'", txn_type));
    }
    if let Some(ref desc) = update_data.description {
        updates.push(format!("description = '{}'", desc));
    }
    if let Some(ref frequency) = update_data.frequency {
        updates.push(format!("frequency = '{}'", frequency));
    }
    if let Some(is_active) = update_data.is_active {
        updates.push(format!("is_active = {}", if is_active { 1 } else { 0 }));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("No fields to update".into()));
    }

    let query = format!(
        "UPDATE recurring_transactions SET {}, updated_at = datetime('now') WHERE id = {}",
        updates.join(", "),
        id
    );

    let result = sqlx::query(&query).execute(pool.get_ref()).await;

    match result {
        Ok(_) => {
            let recurring = sqlx::query_as::<_, RecurringTransaction>(
                "SELECT * FROM recurring_transactions WHERE id = ?",
            )
            .bind(id)
            .fetch_one(pool.get_ref())
            .await
            .unwrap();
            HttpResponse::Ok().json(ApiResponse::success(recurring))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// DELETE /recurring-transactions/{id} - Delete recurring transaction
#[delete("/recurring-transactions/{id}")]
async fn delete_recurring_transaction(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> impl Responder {
    let id = id.into_inner();

    let result = sqlx::query("DELETE FROM recurring_transactions WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok()
                    .json(ApiResponse::success("Recurring transaction deleted successfully"))
            } else {
                HttpResponse::NotFound()
                    .json(ApiResponse::<()>::error("Recurring transaction not found".into()))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// POST /recurring-transactions/process - Process due recurring transactions
#[post("/recurring-transactions/process")]
async fn process_recurring_transactions(pool: web::Data<SqlitePool>) -> impl Responder {
    match recurring::process_due_recurring(pool.get_ref()).await {
        Ok(result) => HttpResponse::Ok().json(ApiResponse::success(format!(
            "Processed {} recurring transactions, created {} new transactions",
            result.due, result.created
        ))),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

// ============================================================================
// Analytics & Insights Endpoints
// ============================================================================

/// GET /analytics/spending-by-category - Get spending breakdown by category
#[get("/analytics/spending-by-category")]
async fn get_spending_by_category(
    pool: web::Data<SqlitePool>,
    query: web::Query<AnalyticsFilter>,
) -> impl Responder {
    let mut where_clauses = vec!["t.transaction_type = 'expense'".to_string()];

    if let Some(user_id) = query.user_id {
        where_clauses.push(format!(
            "t.account_id IN (SELECT id FROM accounts WHERE user_id = {})",
            user_id
        ));
    }
    if let Some(ref start_date) = query.start_date {
        where_clauses.push(format!("t.transaction_date >= '{}'", start_date));
    }
    if let Some(ref end_date) = query.end_date {
        where_clauses.push(format!("t.transaction_date <= '{}'", end_date));
    }

    let where_sql = format!("WHERE {}", where_clauses.join(" AND "));

    let query_sql = format!(
        "SELECT c.id as category_id, c.name as category_name, 
                SUM(ABS(tc.amount)) as total_amount, COUNT(DISTINCT t.id) as transaction_count
         FROM transactions t
         JOIN transaction_categories tc ON t.id = tc.transaction_id
         JOIN categories c ON tc.category_id = c.id
         {}
         GROUP BY c.id, c.name
         ORDER BY total_amount DESC",
        where_sql
    );

    let results = sqlx::query_as::<_, CategorySpendingSummary>(&query_sql)
        .fetch_all(pool.get_ref())
        .await;

    match results {
        Ok(data) => HttpResponse::Ok().json(ApiResponse::success(data)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /analytics/monthly-summary - Get monthly income/expense summary
#[get("/analytics/monthly-summary")]
async fn get_monthly_summary(
    pool: web::Data<SqlitePool>,
    query: web::Query<AnalyticsFilter>,
) -> impl Responder {
    let mut where_clauses = Vec::new();

    if let Some(user_id) = query.user_id {
        where_clauses.push(format!(
            "account_id IN (SELECT id FROM accounts WHERE user_id = {})",
            user_id
        ));
    }
    if let Some(ref start_date) = query.start_date {
        where_clauses.push(format!("transaction_date >= '{}'", start_date));
    }
    if let Some(ref end_date) = query.end_date {
        where_clauses.push(format!("transaction_date <= '{}'", end_date));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let query_sql = format!(
        "SELECT strftime('%Y-%m', transaction_date) as month,
                SUM(CASE WHEN transaction_type = 'income' THEN amount ELSE 0 END) as total_income,
                SUM(CASE WHEN transaction_type = 'expense' THEN ABS(amount) ELSE 0 END) as total_expense,
                SUM(CASE WHEN transaction_type = 'income' THEN amount ELSE -ABS(amount) END) as net_change,
                COUNT(*) as transaction_count
         FROM transactions
         {}
         GROUP BY strftime('%Y-%m', transaction_date)
         ORDER BY month DESC
         LIMIT 12",
        where_sql
    );

    let results = sqlx::query_as::<_, MonthlySummary>(&query_sql)
        .fetch_all(pool.get_ref())
        .await;

    match results {
        Ok(data) => HttpResponse::Ok().json(ApiResponse::success(data)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /analytics/spending-comparison - Compare spending between periods
#[get("/analytics/spending-comparison")]
async fn get_spending_comparison(
    pool: web::Data<SqlitePool>,
    query: web::Query<SpendingComparisonQuery>,
) -> impl Responder {
    let mut user_filter = String::new();
    if let Some(user_id) = query.user_id {
        user_filter = format!(
            "AND account_id IN (SELECT id FROM accounts WHERE user_id = {})",
            user_id
        );
    }

    // Get current period spending
    let current_sql = format!(
        "SELECT SUM(ABS(amount)) as total
         FROM transactions
         WHERE transaction_type = 'expense'
         AND transaction_date >= ? AND transaction_date <= ?
         {}",
        user_filter
    );

    let current_total: Option<f64> = sqlx::query_scalar(&current_sql)
        .bind(&query.current_start)
        .bind(&query.current_end)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap_or(None);

    // Get previous period spending
    let previous_sql = format!(
        "SELECT SUM(ABS(amount)) as total
         FROM transactions
         WHERE transaction_type = 'expense'
         AND transaction_date >= ? AND transaction_date <= ?
         {}",
        user_filter
    );

    let previous_total: Option<f64> = sqlx::query_scalar(&previous_sql)
        .bind(&query.previous_start)
        .bind(&query.previous_end)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap_or(None);

    let current = current_total.unwrap_or(0.0);
    let previous = previous_total.unwrap_or(0.0);
    let change_amount = current - previous;
    let change_percentage = if previous > 0.0 {
        (change_amount / previous) * 100.0
    } else {
        0.0
    };

    let comparison = SpendingComparison {
        current_period_total: current,
        previous_period_total: previous,
        change_amount,
        change_percentage,
    };

    HttpResponse::Ok().json(ApiResponse::success(comparison))
}

/// GET /analytics/top-categories - Get top spending categories
#[get("/analytics/top-categories")]
async fn get_top_categories(
    pool: web::Data<SqlitePool>,
    query: web::Query<AnalyticsFilter>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(5);
    let mut where_clauses = vec!["t.transaction_type = 'expense'".to_string()];

    if let Some(user_id) = query.user_id {
        where_clauses.push(format!(
            "t.account_id IN (SELECT id FROM accounts WHERE user_id = {})",
            user_id
        ));
    }
    if let Some(ref start_date) = query.start_date {
        where_clauses.push(format!("t.transaction_date >= '{}'", start_date));
    }
    if let Some(ref end_date) = query.end_date {
        where_clauses.push(format!("t.transaction_date <= '{}'", end_date));
    }

    let where_sql = format!("WHERE {}", where_clauses.join(" AND "));

    let query_sql = format!(
        "SELECT c.id as category_id, c.name as category_name,
                SUM(ABS(tc.amount)) as total_amount, COUNT(DISTINCT t.id) as transaction_count
         FROM transactions t
         JOIN transaction_categories tc ON t.id = tc.transaction_id
         JOIN categories c ON tc.category_id = c.id
         {}
         GROUP BY c.id, c.name
         ORDER BY total_amount DESC
         LIMIT {}",
        where_sql, limit
    );

    let results = sqlx::query_as::<_, CategorySpendingSummary>(&query_sql)
        .fetch_all(pool.get_ref())
        .await;

    match results {
        Ok(data) => HttpResponse::Ok().json(ApiResponse::success(data)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

// ============================================================================
// Data Export Endpoints
// ============================================================================

/// GET /export/transactions/csv - Export transactions as CSV
#[get("/export/transactions/csv")]
async fn export_transactions_csv(
    pool: web::Data<SqlitePool>,
    query: web::Query<ExportFilter>,
) -> impl Responder {
    let mut where_clauses = Vec::new();

    if let Some(user_id) = query.user_id {
        where_clauses.push(format!(
            "t.account_id IN (SELECT id FROM accounts WHERE user_id = {})",
            user_id
        ));
    }
    if let Some(ref start_date) = query.start_date {
        where_clauses.push(format!("t.transaction_date >= '{}'", start_date));
    }
    if let Some(ref end_date) = query.end_date {
        where_clauses.push(format!("t.transaction_date <= '{}'", end_date));
    }
    if let Some(account_id) = query.account_id {
        where_clauses.push(format!("t.account_id = {}", account_id));
    }
    if let Some(category_id) = query.category_id {
        where_clauses.push(format!(
            "t.id IN (SELECT transaction_id FROM transaction_categories WHERE category_id = {})",
            category_id
        ));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let query_sql = format!(
        "SELECT t.id, t.account_id, a.name as account_name, t.amount, t.transaction_type,
                t.description, t.transaction_date, a.currency
         FROM transactions t
         JOIN accounts a ON t.account_id = a.id
         {}
         ORDER BY t.transaction_date DESC",
        where_sql
    );

    let rows = sqlx::query(&query_sql)
        .fetch_all(pool.get_ref())
        .await;

    match rows {
        Ok(rows) => {
            use sqlx::Row;
            let mut csv = String::from("id,account_id,account_name,amount,type,description,date,currency\n");

            for row in rows {
                let id: i64 = row.get("id");
                let account_id: i64 = row.get("account_id");
                let account_name: String = row.get("account_name");
                let amount: f64 = row.get("amount");
                let txn_type: String = row.get("transaction_type");
                let description: Option<String> = row.get("description");
                let date: chrono::DateTime<Utc> = row.get("transaction_date");
                let currency: String = row.get("currency");

                csv.push_str(&format!(
                    "{},{},\"{}\",{:.2},{},\"{}\",{},{}\n",
                    id,
                    account_id,
                    account_name.replace("\"", "\"\""),
                    amount,
                    txn_type,
                    description.unwrap_or_default().replace("\"", "\"\""),
                    date.format("%Y-%m-%d %H:%M:%S"),
                    currency
                ));
            }

            HttpResponse::Ok()
                .content_type("text/csv")
                .insert_header(("Content-Disposition", "attachment; filename=\"transactions.csv\""))
                .body(csv)
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /export/transactions/json - Export transactions as JSON
#[get("/export/transactions/json")]
async fn export_transactions_json(
    pool: web::Data<SqlitePool>,
    query: web::Query<ExportFilter>,
) -> impl Responder {
    let mut where_clauses = Vec::new();

    if let Some(user_id) = query.user_id {
        where_clauses.push(format!(
            "account_id IN (SELECT id FROM accounts WHERE user_id = {})",
            user_id
        ));
    }
    if let Some(ref start_date) = query.start_date {
        where_clauses.push(format!("transaction_date >= '{}'", start_date));
    }
    if let Some(ref end_date) = query.end_date {
        where_clauses.push(format!("transaction_date <= '{}'", end_date));
    }
    if let Some(account_id) = query.account_id {
        where_clauses.push(format!("account_id = {}", account_id));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let query_sql = format!(
        "SELECT * FROM transactions {} ORDER BY transaction_date DESC",
        where_sql
    );

    let transactions = sqlx::query_as::<_, Transaction>(&query_sql)
        .fetch_all(pool.get_ref())
        .await;

    match transactions {
        Ok(data) => {
            let json = serde_json::to_string_pretty(&data).unwrap_or_default();
            HttpResponse::Ok()
                .content_type("application/json")
                .insert_header(("Content-Disposition", "attachment; filename=\"transactions.json\""))
                .body(json)
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /export/accounts/csv - Export accounts as CSV
#[get("/export/accounts/csv")]
async fn export_accounts_csv(
    pool: web::Data<SqlitePool>,
    query: web::Query<ExportFilter>,
) -> impl Responder {
    let mut where_clauses = Vec::new();

    if let Some(user_id) = query.user_id {
        where_clauses.push(format!("user_id = {}", user_id));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let query_sql = format!(
        "SELECT * FROM accounts {} ORDER BY name",
        where_sql
    );

    let accounts = sqlx::query_as::<_, Account>(&query_sql)
        .fetch_all(pool.get_ref())
        .await;

    match accounts {
        Ok(accounts) => {
            let mut csv = String::from("id,user_id,name,type,bank_name,currency,initial_balance,current_balance,created_at\n");

            for a in accounts {
                csv.push_str(&format!(
                    "{},{},\"{}\",{},\"{}\",{},{:.2},{:.2},{}\n",
                    a.id,
                    a.user_id,
                    a.name.replace("\"", "\"\""),
                    a.account_type,
                    a.bank_name.unwrap_or_default().replace("\"", "\"\""),
                    a.currency,
                    a.initial_balance,
                    a.current_balance,
                    a.created_at.format("%Y-%m-%d %H:%M:%S")
                ));
            }

            HttpResponse::Ok()
                .content_type("text/csv")
                .insert_header(("Content-Disposition", "attachment; filename=\"accounts.csv\""))
                .body(csv)
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string())),
    }
}

/// GET /export/summary/json - Export complete financial summary as JSON
#[get("/export/summary/json")]
async fn export_summary_json(
    pool: web::Data<SqlitePool>,
    query: web::Query<ExportFilter>,
) -> impl Responder {
    let user_filter = if let Some(user_id) = query.user_id {
        format!("WHERE user_id = {}", user_id)
    } else {
        String::new()
    };

    // Get accounts
    let accounts_sql = format!("SELECT * FROM accounts {}", user_filter);
    let accounts = sqlx::query_as::<_, Account>(&accounts_sql)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default();

    // Get categories
    let categories_sql = format!("SELECT * FROM categories {}", user_filter);
    let categories = sqlx::query_as::<_, Category>(&categories_sql)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_default();

    // Get transactions for user's accounts
    let account_ids: Vec<i64> = accounts.iter().map(|a| a.id).collect();
    let transactions = if !account_ids.is_empty() {
        let placeholders = account_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let txn_sql = format!(
            "SELECT * FROM transactions WHERE account_id IN ({}) ORDER BY transaction_date DESC",
            placeholders
        );
        let mut q = sqlx::query_as::<_, Transaction>(&txn_sql);
        for id in &account_ids {
            q = q.bind(id);
        }
        q.fetch_all(pool.get_ref()).await.unwrap_or_default()
    } else {
        Vec::new()
    };

    let summary = FinancialExportSummary {
        export_date: Utc::now(),
        accounts,
        categories,
        transactions,
    };

    let json = serde_json::to_string_pretty(&summary).unwrap_or_default();
    HttpResponse::Ok()
        .content_type("application/json")
        .insert_header(("Content-Disposition", "attachment; filename=\"financial_summary.json\""))
        .body(json)
}

// ============================================================================
// Configuration
// ============================================================================

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users)
        .service(get_user)
        .service(create_user)
        .service(update_user)
        .service(delete_user)
        .service(get_accounts)
        .service(get_account)
        .service(create_account)
        .service(update_account)
        .service(delete_account)
        .service(get_categories)
        .service(get_category)
        .service(create_category)
        .service(update_category)
        .service(delete_category)
        .service(get_transactions)
        .service(get_transaction)
        .service(create_transaction)
        .service(update_transaction)
        .service(delete_transaction)
        .service(get_exchange_rates)
        .service(get_latest_rates)
        .service(convert_currency)
        .service(create_exchange_rate)
        .service(update_exchange_rate)
        .service(delete_rates_bulk)
        .service(delete_exchange_rate)
        .service(get_exchange_rate)
        // Recurring transactions
        .service(get_recurring_transactions)
        .service(get_recurring_transaction)
        .service(create_recurring_transaction)
        .service(update_recurring_transaction)
        .service(delete_recurring_transaction)
        .service(process_recurring_transactions)
        // Analytics
        .service(get_spending_by_category)
        .service(get_monthly_summary)
        .service(get_spending_comparison)
        .service(get_top_categories)
        // Export
        .service(export_transactions_csv)
        .service(export_transactions_json)
        .service(export_accounts_csv)
        .service(export_summary_json);
}
