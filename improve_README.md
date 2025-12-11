# Personal Finance Tracker - Feature Improvements

This document details all the new features and improvements added to the Personal Finance Tracker application.

## Table of Contents

1. [Overview](#overview)
2. [New API Endpoints](#new-api-endpoints)
   - [Recurring Transactions](#recurring-transactions-api)
   - [Analytics & Insights](#analytics--insights-api)
   - [Data Export](#data-export-api)
3. [TUI Improvements](#tui-improvements)
4. [New Data Models](#new-data-models)
5. [Usage Examples](#usage-examples)

---

## Overview

The following features have been implemented based on the project proposal:

| Feature | Section | Status |
|---------|---------|--------|
| Recurring Transactions | 2.2.3 | ✅ Complete |
| Expense Analysis & Insights | 2.2.4 | ✅ Complete |
| Data Export (CSV/JSON) | 2.2.5 | ✅ Complete |
| TUI Enhancements | 2.2.6 | ✅ Complete |
| Multi-Currency Support | 2.2.2 | ✅ Complete |

### Bug Fixes & Enhancements (Latest Update)

| Fix | Description |
|-----|-------------|
| Category Spending Display | Fixed the "Top Spending Categories" in Reports to correctly show spending data using INNER JOINs |
| Currency Filter | Added currency filtering for transactions view (press `f` on Transactions screen) |
| Currency Display | Transactions now show their currency (from the associated account) |
| Add Transaction Form | Now shows available accounts with their currencies for easy selection |
| **List Scrolling** | All lists now properly scroll - you can view all items using ↑↓/PgUp/PgDn/Home/End |
| **Position Indicator** | Lists show current position (e.g., `[5/100]`) in the title bar |

---

## New API Endpoints

### Recurring Transactions API

Manage recurring transactions like rent, salary, and subscriptions.

#### List Recurring Transactions

```http
GET /recurring-transactions
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `account_id` | integer | Filter by account ID |
| `is_active` | boolean | Filter by active status |
| `frequency` | string | Filter by frequency (daily/weekly/monthly/yearly) |
| `page` | integer | Page number (default: 1) |
| `page_size` | integer | Items per page (default: 20) |

**Example:**
```bash
curl "http://localhost:8080/recurring-transactions?is_active=true&frequency=monthly"
```

#### Get Single Recurring Transaction

```http
GET /recurring-transactions/{id}
```

**Example:**
```bash
curl "http://localhost:8080/recurring-transactions/1"
```

#### Create Recurring Transaction

```http
POST /recurring-transactions
```

**Request Body:**
```json
{
  "account_id": 1,
  "category_id": 11,
  "amount": -1500.00,
  "transaction_type": "expense",
  "description": "Monthly rent payment",
  "frequency": "monthly",
  "start_date": "2024-01-01T00:00:00Z",
  "end_date": null
}
```

**Fields:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `account_id` | integer | Yes | Account to charge/credit |
| `category_id` | integer | No | Category for the transaction |
| `amount` | float | Yes | Amount (positive for income, negative for expense) |
| `transaction_type` | string | Yes | "income" or "expense" |
| `description` | string | No | Description of the transaction |
| `frequency` | string | Yes | "daily", "weekly", "monthly", or "yearly" |
| `start_date` | datetime | Yes | When the recurring transaction starts |
| `end_date` | datetime | No | When to stop (null = never) |

#### Update Recurring Transaction

```http
PUT /recurring-transactions/{id}
```

**Request Body:**
```json
{
  "amount": -1600.00,
  "description": "Updated rent amount",
  "is_active": true
}
```

#### Delete Recurring Transaction

```http
DELETE /recurring-transactions/{id}
```

#### Process Due Recurring Transactions

Automatically creates transactions for all due recurring transactions.

```http
POST /recurring-transactions/process
```

**Example:**
```bash
curl -X POST "http://localhost:8080/recurring-transactions/process"
```

**Response:**
```json
{
  "success": true,
  "data": "Processed 5 recurring transactions, created 5 new transactions"
}
```

---

### Analytics & Insights API

Get spending insights and financial summaries.

#### Spending by Category

Get a breakdown of spending by category.

```http
GET /analytics/spending-by-category
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `user_id` | integer | Filter by user ID |
| `start_date` | datetime | Start of date range |
| `end_date` | datetime | End of date range |

**Example:**
```bash
curl "http://localhost:8080/analytics/spending-by-category?user_id=1"
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "category_id": 11,
      "category_name": "Rent",
      "total_amount": 3000.00,
      "transaction_count": 2
    },
    {
      "category_id": 6,
      "category_name": "Groceries",
      "total_amount": 350.00,
      "transaction_count": 3
    }
  ]
}
```

#### Monthly Summary

Get income/expense summary by month.

```http
GET /analytics/monthly-summary
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `user_id` | integer | Filter by user ID |
| `start_date` | datetime | Start of date range |
| `end_date` | datetime | End of date range |

**Example:**
```bash
curl "http://localhost:8080/analytics/monthly-summary?user_id=1"
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "month": "2024-12",
      "total_income": 5000.00,
      "total_expense": 2500.00,
      "net_change": 2500.00,
      "transaction_count": 15
    },
    {
      "month": "2024-11",
      "total_income": 5500.00,
      "total_expense": 2800.00,
      "net_change": 2700.00,
      "transaction_count": 18
    }
  ]
}
```

#### Spending Comparison

Compare spending between two time periods.

```http
GET /analytics/spending-comparison
```

**Query Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | integer | No | Filter by user ID |
| `current_start` | datetime | Yes | Start of current period |
| `current_end` | datetime | Yes | End of current period |
| `previous_start` | datetime | Yes | Start of previous period |
| `previous_end` | datetime | Yes | End of previous period |

**Example:**
```bash
curl "http://localhost:8080/analytics/spending-comparison?user_id=1&current_start=2024-12-01T00:00:00Z&current_end=2024-12-31T23:59:59Z&previous_start=2024-11-01T00:00:00Z&previous_end=2024-11-30T23:59:59Z"
```

**Response:**
```json
{
  "success": true,
  "data": {
    "current_period_total": 2500.00,
    "previous_period_total": 2800.00,
    "change_amount": -300.00,
    "change_percentage": -10.71
  }
}
```

#### Top Spending Categories

Get the top spending categories.

```http
GET /analytics/top-categories
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `user_id` | integer | Filter by user ID |
| `start_date` | datetime | Start of date range |
| `end_date` | datetime | End of date range |
| `limit` | integer | Number of categories to return (default: 5) |

**Example:**
```bash
curl "http://localhost:8080/analytics/top-categories?user_id=1&limit=3"
```

---

### Data Export API

Export financial data in CSV or JSON format.

#### Export Transactions as CSV

```http
GET /export/transactions/csv
```

**Query Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `user_id` | integer | Filter by user ID |
| `start_date` | datetime | Start of date range |
| `end_date` | datetime | End of date range |
| `account_id` | integer | Filter by account ID |
| `category_id` | integer | Filter by category ID |

**Example:**
```bash
curl "http://localhost:8080/export/transactions/csv?user_id=1" -o transactions.csv
```

**Output Format:**
```csv
id,account_id,account_name,amount,type,description,date,currency
1,1,Chase Checking,5000.00,income,"Monthly salary",2024-12-01 00:00:00,USD
2,1,Chase Checking,-1500.00,expense,"Monthly rent",2024-12-01 00:00:00,USD
```

#### Export Transactions as JSON

```http
GET /export/transactions/json
```

**Example:**
```bash
curl "http://localhost:8080/export/transactions/json?user_id=1" -o transactions.json
```

#### Export Accounts as CSV

```http
GET /export/accounts/csv
```

**Example:**
```bash
curl "http://localhost:8080/export/accounts/csv?user_id=1" -o accounts.csv
```

**Output Format:**
```csv
id,user_id,name,type,bank_name,currency,initial_balance,current_balance,created_at
1,1,"Chase Checking",checking,"Chase Bank",USD,5000.00,4500.00,2024-01-01 00:00:00
```

#### Export Full Financial Summary

```http
GET /export/summary/json
```

Exports a complete financial summary including accounts, transactions, and categories.

**Example:**
```bash
curl "http://localhost:8080/export/summary/json?user_id=1" -o financial_summary.json
```

**Response Structure:**
```json
{
  "export_date": "2024-12-11T10:30:00Z",
  "accounts": [...],
  "categories": [...],
  "transactions": [...]
}
```

---

## TUI Improvements

The Text User Interface has been enhanced with new screens and features.

### New Navigation

The TUI now has **8 tabs** accessible via number keys or arrow keys:

| Key | Tab | Description |
|-----|-----|-------------|
| 1 | Dashboard | Overview with quick stats and recent transactions |
| 2 | Accounts | View all accounts and balances |
| 3 | Transactions | View, add, delete transactions |
| 4 | Categories | View spending categories |
| 5 | **Recurring** | **NEW** - Manage recurring transactions |
| 6 | FX Rates | View and manage exchange rates |
| 7 | Reports | **IMPROVED** - Financial summary with category insights |
| 8 | **Export** | **NEW** - Export data to files |

### Transactions Screen (Tab 3) - Enhanced

**New Features:**
- **Currency Display** - Each transaction now shows its currency (from associated account)
- **Currency Filter** - Filter transactions by currency type
- **Enhanced Add Form** - Shows available accounts with their currencies

**Keyboard Shortcuts:**
| Key | Action |
|-----|--------|
| `a` | Add new transaction (shows accounts with currencies) |
| `f` | **NEW** - Filter by currency |
| `d` | Delete transaction |
| `Enter` | View details |
| `↑/↓` | Navigate list |

**Currency Filter Dialog:**
Press `f` to open the currency filter:
- `0` - Show all currencies
- `1-9` - Select specific currency from list

**Display Format:**
```
2024-12-01 Expense     1500.00 USD  | Monthly rent
2024-12-05 Income      5000.00 USD  | Monthly salary
2024-12-10 Expense       50.00 EUR  | Coffee in Europe
```

### Recurring Transactions Screen (Tab 5)

**Features:**
- View all recurring transactions with status, frequency, amount, and next occurrence
- Color-coded status (Green = Active, Red = Paused)
- Color-coded amounts (Green = Income, Red = Expense)

**Keyboard Shortcuts:**
| Key | Action |
|-----|--------|
| `a` | Add new recurring transaction |
| `p` | Process all due recurring transactions |
| `t` | Toggle active/paused status |
| `d` | Delete recurring transaction |
| `Enter` | View details |
| `↑/↓` | Navigate list |

### Improved Reports Screen (Tab 7)

**New Features:**
- **Top Spending Categories** - Visual bar chart showing spending distribution
- Percentage breakdown for each category
- Transaction counts per category

**Display:**
```
╔══════════════════════════════════════════════════════════════╗
║ Top Spending Categories (Insights)                            ║
╠══════════════════════════════════════════════════════════════╣
║ 1. Rent                 $1500.00 ( 45.5%) ████████████       ║
║ 2. Groceries            $ 350.00 ( 10.6%) ██                 ║
║ 3. Dining Out           $ 180.00 (  5.5%) █                  ║
╚══════════════════════════════════════════════════════════════╝
```

### Export Screen (Tab 8)

**Features:**
- View summary of data available for export
- Export to multiple formats

**Keyboard Shortcuts:**
| Key | Action |
|-----|--------|
| `e` | Open export dialog |
| `1` | Export transactions as CSV |
| `2` | Export transactions as JSON |
| `3` | Export accounts as CSV |
| `4` | Export full summary as JSON |

**Exported files are saved to the current working directory:**
- `transactions_export.csv`
- `transactions_export.json`
- `accounts_export.csv`
- `financial_summary.json`

### General Keyboard Shortcuts

| Key | Action | Available On |
|-----|--------|--------------|
| `1-8` | Switch to tab 1-8 | All screens |
| `←/→` | Navigate between tabs | All screens |
| `↑/↓` | Navigate within lists (one item) | All screens |
| `PgUp/PgDn` | **Scroll 10 items at a time** | **All list screens (NEW)** |
| `Home` | **Jump to first item** | **All list screens (NEW)** |
| `End` | **Jump to last item** | **All list screens (NEW)** |
| `a` | Add new item | Transactions, Recurring, Exchange Rates |
| `d` | Delete selected item | Transactions, Recurring, Exchange Rates |
| `f` | **Filter by currency** | **Transactions (NEW)** |
| `c` | Convert currency | Exchange Rates |
| `p` | Process due recurring | Recurring Transactions |
| `t` | Toggle active/paused | Recurring Transactions |
| `e` | Export data | Export screen |
| `r` | Refresh data | All screens |
| `u` | Switch user | All screens |
| `q` | Quit application | All screens |
| `Esc` | Cancel/Go back | All screens |

---

## New Data Models

The following models were added to `models.rs`:

### RecurringTransactionFilter

```rust
pub struct RecurringTransactionFilter {
    pub account_id: Option<i64>,
    pub is_active: Option<bool>,
    pub frequency: Option<String>,
    pub page: i64,
    pub page_size: i64,
}
```

### AnalyticsFilter

```rust
pub struct AnalyticsFilter {
    pub user_id: Option<i64>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}
```

### SpendingComparisonQuery

```rust
pub struct SpendingComparisonQuery {
    pub user_id: Option<i64>,
    pub current_start: DateTime<Utc>,
    pub current_end: DateTime<Utc>,
    pub previous_start: DateTime<Utc>,
    pub previous_end: DateTime<Utc>,
}
```

### SpendingComparison

```rust
pub struct SpendingComparison {
    pub current_period_total: f64,
    pub previous_period_total: f64,
    pub change_amount: f64,
    pub change_percentage: f64,
}
```

### ExportFilter

```rust
pub struct ExportFilter {
    pub user_id: Option<i64>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub account_id: Option<i64>,
    pub category_id: Option<i64>,
}
```

### FinancialExportSummary

```rust
pub struct FinancialExportSummary {
    pub export_date: DateTime<Utc>,
    pub accounts: Vec<Account>,
    pub categories: Vec<Category>,
    pub transactions: Vec<Transaction>,
}
```

---

## Usage Examples

### Running the Application

**Start the REST API Server:**
```bash
cargo run serve
```

**Launch the TUI:**
```bash
cargo run tui
```

### Sample API Workflow

1. **Create a recurring expense:**
```bash
curl -X POST http://localhost:8080/recurring-transactions \
  -H "Content-Type: application/json" \
  -d '{
    "account_id": 1,
    "category_id": 11,
    "amount": -1500,
    "transaction_type": "expense",
    "description": "Monthly rent",
    "frequency": "monthly",
    "start_date": "2024-01-01T00:00:00Z"
  }'
```

2. **Process due recurring transactions:**
```bash
curl -X POST http://localhost:8080/recurring-transactions/process
```

3. **View spending insights:**
```bash
curl "http://localhost:8080/analytics/spending-by-category?user_id=1"
```

4. **Compare this month vs last month:**
```bash
curl "http://localhost:8080/analytics/spending-comparison?user_id=1&current_start=2024-12-01T00:00:00Z&current_end=2024-12-31T23:59:59Z&previous_start=2024-11-01T00:00:00Z&previous_end=2024-11-30T23:59:59Z"
```

5. **Export data for tax preparation:**
```bash
curl "http://localhost:8080/export/transactions/csv?user_id=1&start_date=2024-01-01T00:00:00Z&end_date=2024-12-31T23:59:59Z" -o 2024_transactions.csv
```

---

## File Changes Summary

| File | Changes |
|------|---------|
| `src/api.rs` | Added 15+ new endpoints for recurring transactions, analytics, and export |
| `src/models.rs` | Added 6 new data models with `FromRow` derive for database queries |
| `src/tui.rs` | Added 2 new screens, improved reports, new keyboard handlers |
| `src/main.rs` | Updated API documentation in server startup message |

---

## Future Improvements

Potential enhancements for future development:

1. **Budget Tracking** - Set and monitor spending budgets per category
2. **Email Notifications** - Alert when recurring transactions are processed
3. **Charts and Graphs** - Add visual charts in the TUI using sparklines
4. **Import Functionality** - Import transactions from bank CSV exports
5. **Multi-currency Reports** - Aggregate reports across different currencies
6. **Scheduled Processing** - Automatic background processing of recurring transactions

---

*Last Updated: December 2024*

