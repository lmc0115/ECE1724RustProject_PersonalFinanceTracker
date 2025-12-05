use crate::models::*;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::{TimeZone, Utc};
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
    let offset = (query.pagination.page - 1) * query.pagination.page_size;

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
        where_sql, query.pagination.page_size, offset
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
                page: query.pagination.page,
                page_size: query.pagination.page_size,
                total_pages: (total + query.pagination.page_size - 1) / query.pagination.page_size,
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
async fn delete_transaction(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> impl Responder {
    let id = id.into_inner();

    // 1. Fetch the transaction so we know its amount, type, and account
    let existing_txn =
        sqlx::query_as::<_, Transaction>("SELECT * FROM transactions WHERE id = ?")
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
    if let Err(e) = sqlx::query(
        "DELETE FROM transaction_categories WHERE transaction_id = ?",
    )
    .bind(id)
    .execute(pool.get_ref())
    .await
    {
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<()>::error(e.to_string()));
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

                HttpResponse::Ok().json(ApiResponse::success(
                    "Transaction deleted successfully",
                ))
            } else {
                // Shouldn’t really happen since we already fetched it,
                // but keep the check for safety.
                HttpResponse::NotFound()
                    .json(ApiResponse::<()>::error("Transaction not found".into()))
            }
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<()>::error(e.to_string())),
    }
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
        .service(delete_transaction);
}
