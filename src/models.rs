use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// User Models
// ============================================================================

/// User entity - represents a user account
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)] // Don't expose password hash in JSON responses
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data required to create a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String, // Plain text password (will be hashed before storage)
}

/// Data for updating a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>, // Plain text password (will be hashed before storage)
}

// ============================================================================
// Account Models
// ============================================================================

/// Account entity - represents a bank account
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub account_type: String, // "checking", "savings", "credit_card"
    pub bank_name: Option<String>,
    pub currency: String, // ISO 4217 currency code (e.g., "USD", "EUR")
    pub initial_balance: f64,
    pub current_balance: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data required to create a new account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccount {
    pub user_id: i64,
    pub name: String,
    pub account_type: String, // "checking", "savings", "credit_card"
    pub bank_name: Option<String>,
    pub currency: Option<String>, // Defaults to "USD" if not provided
    pub initial_balance: Option<f64>, // Defaults to 0.0 if not provided
}

/// Data for updating an account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub account_type: Option<String>,
    pub bank_name: Option<String>,
    pub currency: Option<String>,
}

// ============================================================================
// Category Models
// ============================================================================

/// Category entity - represents a transaction category
/// Note: Categories are now type-agnostic (no category_type field)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data required to create a new category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategory {
    pub user_id: i64,
    pub name: String,
}

/// Data for updating a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategory {
    pub name: Option<String>,
}

// ============================================================================
// Transaction Models
// ============================================================================

/// Transaction entity - represents a financial transaction
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub account_id: i64,
    pub amount: f64,
    pub transaction_type: String, // "income", "expense", "transfer"
    pub description: Option<String>,
    pub transaction_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data required to create a new transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransaction {
    pub account_id: i64,
    pub amount: f64,
    pub transaction_type: String, // "income", "expense", "transfer"
    pub description: Option<String>,
    pub transaction_date: Option<DateTime<Utc>>, // Defaults to now if not provided
    pub categories: Vec<CategoryAmount>,         // For split transactions
}

/// Category amount for split transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAmount {
    pub category_id: i64,
    pub amount: f64,
}

/// Data for updating a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransaction {
    pub amount: Option<f64>,
    pub transaction_type: Option<String>,
    pub description: Option<String>,
    pub transaction_date: Option<DateTime<Utc>>,
}

/// Transaction with its categories (joined data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionWithCategories {
    #[serde(flatten)]
    pub transaction: Transaction,
    pub categories: Vec<TransactionCategoryDetail>,
}

/// Category detail for a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionCategoryDetail {
    pub category_id: i64,
    pub category_name: String,
    pub amount: f64,
}

// ============================================================================
// Transaction_Categories Models
// ============================================================================

/// Transaction_Category entity - links transactions to categories
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TransactionCategory {
    pub id: i64,
    pub transaction_id: i64,
    pub category_id: i64,
    pub amount: f64,
}

/// Data required to create a new transaction-category link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionCategory {
    pub transaction_id: i64,
    pub category_id: i64,
    pub amount: f64,
}

// ============================================================================
// Recurring_Transactions Models
// ============================================================================

/// Recurring_Transaction entity - represents a recurring transaction template
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RecurringTransaction {
    pub id: i64,
    pub account_id: i64,
    pub category_id: Option<i64>,
    pub amount: f64,
    pub transaction_type: String, // "income", "expense"
    pub description: Option<String>,
    pub frequency: String, // "daily", "weekly", "monthly", "yearly"
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub next_occurrence: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data required to create a new recurring transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecurringTransaction {
    pub account_id: i64,
    pub category_id: Option<i64>,
    pub amount: f64,
    pub transaction_type: String, // "income", "expense"
    pub description: Option<String>,
    pub frequency: String, // "daily", "weekly", "monthly", "yearly"
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Data for updating a recurring transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecurringTransaction {
    pub category_id: Option<i64>,
    pub amount: Option<f64>,
    pub transaction_type: Option<String>,
    pub description: Option<String>,
    pub frequency: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

// ============================================================================
// Exchange_Rates Models
// ============================================================================

/// Exchange_Rate entity - represents a currency exchange rate
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub id: i64,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub rate_date: DateTime<Utc>,
    pub source: String, // "api", "bank", "manual", "scraper"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data required to create a new exchange rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExchangeRate {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub rate_date: Option<DateTime<Utc>>,
    pub source: Option<String>, // "api", "bank", "manual", "scraper"
}

/// Data for updating an exchange rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExchangeRate {
    pub rate: Option<f64>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRateFilter {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    pub from_currency: Option<String>,
    pub to_currency: Option<String>,
    pub source: Option<String>,
    pub date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CurrencyConversion {
    pub from_currency: String,
    pub to_currency: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct ConversionResult {
    pub from_currency: String,
    pub to_currency: String,
    pub amount: f64,
    pub rate: f64,
    pub converted_amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteParams {
    pub from_currency: Option<String>,
    pub date: Option<chrono::NaiveDate>,
    pub source: Option<String>,
}

// ============================================================================
// Enums for Type Safety
// ============================================================================

/// Account types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Checking,
    Savings,
    CreditCard,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountType::Checking => "checking",
            AccountType::Savings => "savings",
            AccountType::CreditCard => "credit_card",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "checking" => Some(AccountType::Checking),
            "savings" => Some(AccountType::Savings),
            "credit_card" => Some(AccountType::CreditCard),
            _ => None,
        }
    }
}

/// Transaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Income,
    Expense,
    Transfer,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Income => "income",
            TransactionType::Expense => "expense",
            TransactionType::Transfer => "transfer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "income" => Some(TransactionType::Income),
            "expense" => Some(TransactionType::Expense),
            "transfer" => Some(TransactionType::Transfer),
            _ => None,
        }
    }
}

/// Frequency types for recurring transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Frequency {
    pub fn as_str(&self) -> &'static str {
        match self {
            Frequency::Daily => "daily",
            Frequency::Weekly => "weekly",
            Frequency::Monthly => "monthly",
            Frequency::Yearly => "yearly",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(Frequency::Daily),
            "weekly" => Some(Frequency::Weekly),
            "monthly" => Some(Frequency::Monthly),
            "yearly" => Some(Frequency::Yearly),
            _ => None,
        }
    }
}

/// Exchange rate source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeRateSource {
    Api,
    Bank,
    Manual,
    Scraper,
}

impl ExchangeRateSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExchangeRateSource::Api => "api",
            ExchangeRateSource::Bank => "bank",
            ExchangeRateSource::Manual => "manual",
            ExchangeRateSource::Scraper => "scraper",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "api" => Some(ExchangeRateSource::Api),
            "bank" => Some(ExchangeRateSource::Bank),
            "manual" => Some(ExchangeRateSource::Manual),
            "scraper" => Some(ExchangeRateSource::Scraper),
            _ => None,
        }
    }
}

// ============================================================================
// Helper Structs for API Responses
// ============================================================================

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

// ============================================================================
// Query Parameters
// ============================================================================

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

/// Transaction filter parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFilter {
    pub account_id: Option<i64>,
    pub transaction_type: Option<String>,
    pub category_id: Option<i64>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

// ============================================================================
// Statistics Models
// ============================================================================

/// Account balance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalanceSummary {
    pub account_id: i64,
    pub account_name: String,
    pub account_type: String,
    pub currency: String,
    pub current_balance: f64,
}

/// Category spending summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySpendingSummary {
    pub category_id: i64,
    pub category_name: String,
    pub total_amount: f64,
    pub transaction_count: i64,
}

/// Monthly summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    pub month: String, // Format: "YYYY-MM"
    pub total_income: f64,
    pub total_expense: f64,
    pub net_change: f64,
    pub transaction_count: i64,
}

/// Currency balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyBalance {
    pub currency: String,
    pub total_balance: f64,
    pub account_count: i64,
}

// ============================================================================
// Validation Helpers
// ============================================================================

impl CreateUser {
    /// Validate user creation data
    pub fn validate(&self) -> Result<(), String> {
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        if self.email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }
        if !self.email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        if self.password.len() < 8 {
            return Err("Password must be at least 8 characters".to_string());
        }
        Ok(())
    }
}

impl CreateAccount {
    /// Validate account creation data
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Account name cannot be empty".to_string());
        }
        if !["checking", "savings", "credit_card"].contains(&self.account_type.as_str()) {
            return Err("Invalid account type".to_string());
        }
        Ok(())
    }
}

impl CreateTransaction {
    /// Validate transaction creation data
    pub fn validate(&self) -> Result<(), String> {
        if !["income", "expense", "transfer"].contains(&self.transaction_type.as_str()) {
            return Err("Invalid transaction type".to_string());
        }

        // Validate category amounts sum equals transaction amount
        if !self.categories.is_empty() {
            let categories_sum: f64 = self.categories.iter().map(|c| c.amount).sum();
            let diff = (self.amount - categories_sum).abs();
            if diff > 0.01 {
                return Err(format!(
                    "Category amounts ({}) must sum to transaction amount ({})",
                    categories_sum, self.amount
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_validation() {
        let valid_user = CreateUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(valid_user.validate().is_ok());

        let invalid_email = CreateUser {
            username: "testuser".to_string(),
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };
        assert!(invalid_email.validate().is_err());

        let short_password = CreateUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "short".to_string(),
        };
        assert!(short_password.validate().is_err());
    }

    #[test]
    fn test_account_type_conversion() {
        assert_eq!(AccountType::Checking.as_str(), "checking");
        assert_eq!(AccountType::from_str("savings"), Some(AccountType::Savings));
        assert_eq!(AccountType::from_str("invalid"), None);
    }

    #[test]
    fn test_transaction_type_conversion() {
        assert_eq!(TransactionType::Income.as_str(), "income");
        assert_eq!(
            TransactionType::from_str("expense"),
            Some(TransactionType::Expense)
        );
        assert_eq!(TransactionType::from_str("invalid"), None);
    }
}
