# Personal Finance Tracker - Complete Testing Guide

This guide walks you through testing all features of the Personal Finance Tracker.

---

## Prerequisites

1. **Build the project:**
   ```powershell
   cd "d:\桌面\textbook\graduate\2025fall\ECE1724\project"
   cargo build
   ```

2. **Ensure database has seed data:**
   ```powershell
   cargo run db_seed
   ```

3. **Check database status:**
   ```powershell
   cargo run db_status
   ```
   You should see counts for users, accounts, transactions, etc.

---

## Part 1: Testing the TUI (Text User Interface)

### Step 1: Launch the TUI
```powershell
cargo run tui
```

### Step 2: User Selection Screen
- You should see "Personal Finance Tracker" title
- A list of users (alice_wang, bob_chen, carol_liu)
- Use `↑` `↓` to highlight different users
- Press `Enter` to select **alice_wang**
- Status message should show "Logged in as alice_wang"

### Step 3: Dashboard (Tab 1)
- Verify you see:
  - Quick Stats (Total Accounts, Total Balance, Income, Expenses, Net Change)
  - Recent Transactions list
- Press `↑` `↓` to scroll through transactions
- Press `Enter` on a transaction to see details
- Press `Esc` to go back

### Step 4: Accounts Screen (Tab 2)
- Press `2` or use `→` to go to Accounts
- Verify you see all accounts with:
  - Account name
  - Bank name
  - Balance with currency (USD, EUR)
- Test scrolling: `↑` `↓` `Page Up` `Page Down` `Home` `End`
- Notice position indicator in title (e.g., `[1/4]`)
- Press `Enter` to view account details
- Press `Esc` to go back

### Step 5: Transactions Screen (Tab 3)
- Press `3` to go to Transactions
- Verify transactions show:
  - Date
  - Type (Income/Expense)
  - Amount
  - Currency (USD/EUR)
  - Description

#### Test Scrolling:
- Press `↓` multiple times - should scroll through all transactions
- Press `Page Down` - should jump 10 items
- Press `End` - should jump to last transaction
- Press `Home` - should jump back to first
- Notice position indicator updates (e.g., `[15/20]`)

#### Test Currency Filter:
- Press `f` to open currency filter dialog
- Press `1` to filter by USD only
- Title should show `[Filter: USD]`
- Only USD transactions should be visible
- Press `f` again, then `0` to show all currencies

#### Test Add Transaction:
- Press `a` to add a new transaction
- You should see:
  - Left panel: Form fields
  - Right panel: Available accounts with their currencies
- Fill in:
  - Account ID: `1` (should show `(USD)` next to it)
  - Amount: `50`
  - Type: `expense`
  - Description: `Test transaction`
  - Category ID: `6` (Groceries)
- Press `Enter` to submit
- Status should show "Transaction added successfully!"
- New transaction should appear in the list

#### Test Delete Transaction:
- Navigate to the test transaction you just created
- Press `d` to delete
- Confirm by pressing `y`
- Transaction should be removed

### Step 6: Categories Screen (Tab 4)
- Press `4` to go to Categories
- Verify categories are displayed in two columns
- Categories include: Salary, Groceries, Rent, etc.

### Step 7: Recurring Transactions Screen (Tab 5)
- Press `5` to go to Recurring
- Verify you see recurring transactions with:
  - Status (Active/Paused in green/red)
  - Frequency (monthly)
  - Amount
  - Next occurrence date
  - Description

#### Test Scrolling:
- Use `↑` `↓` `Page Up` `Page Down` `Home` `End`
- Position indicator should update

#### Test Toggle Active:
- Select a recurring transaction
- Press `t` to toggle active/paused
- Status should change color (Green ↔ Red)
- Press `t` again to toggle back

#### Test Add Recurring:
- Press `a` to add new recurring transaction
- Fill in:
  - Account ID: `1`
  - Amount: `100`
  - Type: `expense`
  - Description: `Test recurring`
  - Category ID: `28` (Subscriptions)
  - Frequency: `monthly`
- Press `Enter` to submit

#### Test Process Due:
- Press `p` to process due recurring transactions
- Status should show how many were processed

#### Test Delete:
- Select the test recurring transaction
- Press `d` then `y` to delete

### Step 8: Exchange Rates Screen (Tab 6)
- Press `6` to go to FX Rates
- Verify you see exchange rates with:
  - From currency (e.g., CAD, USD)
  - To currency (e.g., Euro (EUR))
  - Rate
  - Date
  - Source

#### Test Scrolling (IMPORTANT - This was the bug fix!):
- Press `↓` repeatedly - should scroll through ALL rates
- Press `Page Down` - should jump 10 items
- Press `End` - should jump to last rate
- Press `Home` - should jump to first rate
- Position indicator should show current position (e.g., `[50/100]`)

#### Test Currency Conversion:
- Press `c` to convert currency
- Enter:
  - From Currency: `USD`
  - To Currency: `EUR`
  - Amount: `100`
- Press `Enter`
- Result should show converted amount

#### Test Add Rate:
- Press `a` to add new rate
- Fill in:
  - From Currency: `USD`
  - To Currency: `TEST`
  - Rate: `1.5`
  - Source: `manual`
- Press `Enter`

#### Test Delete Rate:
- Navigate to the TEST rate you created
- Press `d` then `y` to delete

### Step 9: Reports Screen (Tab 7)
- Press `7` to go to Reports
- Verify you see:
  - **Financial Summary** (Income, Expenses, Net Change, Transaction Count)
  - **Top Spending Categories (Insights)** - should show categories with:
    - Rank (1, 2, 3...)
    - Category name
    - Total amount
    - Percentage
    - Visual bar (████)
  - **Account Balances** list

### Step 10: Export Screen (Tab 8)
- Press `8` to go to Export
- Verify you see:
  - Export instructions
  - Data available for export (counts)

#### Test Export:
- Press `e` to open export dialog
- Press `1` to export transactions as CSV
- Status should show "Exported X transactions to transactions_export.csv"
- Check the project folder - file should exist

- Press `e` again
- Press `2` for JSON export
- Press `e` again
- Press `3` for accounts CSV
- Press `e` again
- Press `4` for full summary JSON

### Step 11: User Switch
- Press `u` to switch user
- You should return to user selection screen
- Select a different user (e.g., bob_chen)
- Data should refresh for the new user

### Step 12: Quit
- Press `q` to quit the TUI
- Should return to terminal

---

## Part 2: Testing the REST API

### Step 1: Start the API Server
Open a new terminal:
```powershell
cd "d:\桌面\textbook\graduate\2025fall\ECE1724\project"
cargo run serve
```

Server should start at `http://127.0.0.1:8080`

### Step 2: Test Core Endpoints (using PowerShell)

Open another terminal for testing:

```powershell
# Test Users
Invoke-RestMethod -Uri "http://localhost:8080/users" | ConvertTo-Json

# Test Accounts
Invoke-RestMethod -Uri "http://localhost:8080/accounts" | ConvertTo-Json

# Test Transactions
Invoke-RestMethod -Uri "http://localhost:8080/transactions" | ConvertTo-Json

# Test Categories
Invoke-RestMethod -Uri "http://localhost:8080/categories" | ConvertTo-Json
```

### Step 3: Test Recurring Transactions API

```powershell
# List recurring transactions
Invoke-RestMethod -Uri "http://localhost:8080/recurring-transactions" | ConvertTo-Json

# Get single recurring transaction
Invoke-RestMethod -Uri "http://localhost:8080/recurring-transactions/1" | ConvertTo-Json

# Process due recurring transactions
Invoke-RestMethod -Uri "http://localhost:8080/recurring-transactions/process" -Method POST | ConvertTo-Json
```

### Step 4: Test Analytics API

```powershell
# Spending by category
Invoke-RestMethod -Uri "http://localhost:8080/analytics/spending-by-category?user_id=1" | ConvertTo-Json

# Monthly summary
Invoke-RestMethod -Uri "http://localhost:8080/analytics/monthly-summary?user_id=1" | ConvertTo-Json

# Top categories
Invoke-RestMethod -Uri "http://localhost:8080/analytics/top-categories?user_id=1&limit=5" | ConvertTo-Json
```

### Step 5: Test Export API

```powershell
# Export transactions as CSV
Invoke-WebRequest -Uri "http://localhost:8080/export/transactions/csv?user_id=1" -OutFile "api_transactions.csv"

# Export transactions as JSON
Invoke-WebRequest -Uri "http://localhost:8080/export/transactions/json?user_id=1" -OutFile "api_transactions.json"

# Export accounts as CSV
Invoke-WebRequest -Uri "http://localhost:8080/export/accounts/csv?user_id=1" -OutFile "api_accounts.csv"

# Export full summary
Invoke-WebRequest -Uri "http://localhost:8080/export/summary/json?user_id=1" -OutFile "api_summary.json"
```

### Step 6: Test Exchange Rates API

```powershell
# List exchange rates
Invoke-RestMethod -Uri "http://localhost:8080/exchange-rates" | ConvertTo-Json

# Get latest rates for a currency
Invoke-RestMethod -Uri "http://localhost:8080/exchange-rates/latest/USD" | ConvertTo-Json

# Convert currency
Invoke-RestMethod -Uri "http://localhost:8080/exchange-rates/convert?from=USD&to=EUR&amount=100" | ConvertTo-Json
```

---

## Part 3: Testing Exchange Rate Scraper

```powershell
# Scrape default currencies (CAD, USD, EUR, GBP)
cargo run scrape_rates

# Scrape specific currency
cargo run scrape_rates JPY
```

---

## Quick Checklist

### TUI Features ✓
- [ ] User selection and login
- [ ] Dashboard with stats
- [ ] Accounts list with scrolling
- [ ] Transactions with currency display
- [ ] Transaction currency filter (press `f`)
- [ ] Add transaction with account/currency selection
- [ ] Categories display
- [ ] Recurring transactions CRUD
- [ ] Toggle recurring active/paused
- [ ] Process due recurring
- [ ] Exchange rates with FULL scrolling (PgUp/PgDn/Home/End)
- [ ] Currency conversion
- [ ] Reports with Top Spending Categories (bar chart)
- [ ] Export to CSV/JSON files
- [ ] User switch
- [ ] Position indicator in list titles

### API Features ✓
- [ ] CRUD for Users, Accounts, Categories, Transactions
- [ ] Recurring Transactions CRUD + Process
- [ ] Analytics endpoints (spending, monthly, comparison)
- [ ] Export endpoints (CSV, JSON)
- [ ] Exchange rates CRUD + conversion

---

## Troubleshooting

### "Database is empty" error
Run: `cargo run db_seed`

### TUI not scrolling
Make sure you're using `↑↓` keys, not `j/k` or other vim keys.
Try `Page Down`, `Page Up`, `Home`, `End` for faster navigation.

### Categories not showing in Reports
This was fixed - categories with 0 spending are now filtered out.
If still empty, make sure there are expense transactions linked to categories.

### API server won't start
Check if port 8080 is in use. Kill other processes or change the port in `.env` file:
```
BIND_ADDRESS=127.0.0.1:8081
```

---

*Testing Guide - Last Updated: December 2024*




