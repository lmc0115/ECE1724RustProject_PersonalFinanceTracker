# Personal Finance Tracker API Guide

REST API for managing users, accounts, categories, transactions, recurring transactions, and exchange rates. This guide focuses on how to run the server locally and interact with the endpoints.

## Run the API
- Set environment variable `DATABASE_URL` (e.g., `sqlite://personal-finance-tracker.db`). Optional: `BIND_ADDRESS` (default `127.0.0.1:8080`).
- Seed sample data (optional): `cargo run db_seed`.
- Start the server: `cargo run serve` (listens on `http://127.0.0.1:8080`).

## Data Constraints (DB + validation)
- Users: `username` and `email` must be unique; `password` >= 8 chars; `email` must contain `@`.
- Accounts: `account_type` in `checking`, `savings`, `credit_card`; `user_id` must exist.
- Categories: `(user_id, name)` must be unique.
- Transactions: `transaction_type` in `income`, `expense`, `transfer`; `account_id` must exist; if categories are provided, their amounts must sum to `amount` (±0.01).
- Recurring transactions: `transaction_type` in `income`, `expense`; `frequency` in `daily`, `weekly`, `monthly`, `yearly`; `account_id` must exist.
- Exchange rates: `from_currency` and `to_currency` required; `rate` must be positive; `source` in `api`, `bank`, `manual`, `scraper`.

## Common Workflows (curl examples)

### Create and list users
```bash
# Create (username/email must be unique)
curl -X POST http://127.0.0.1:8080/users \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","password":"password123"}'

# List (pagination defaults: page=1,page_size=20)
curl -s 'http://127.0.0.1:8080/users?page=1&page_size=20'
```

### Create an account for a user and list accounts
```bash
curl -X POST http://127.0.0.1:8080/accounts \
  -H "Content-Type: application/json" \
  -d '{"user_id":1,"name":"Checking","account_type":"checking","currency":"USD","initial_balance":500}'

curl -s 'http://127.0.0.1:8080/accounts?page=1&page_size=20'
```

### Create categories (unique per user) and list
```bash
curl -X POST http://127.0.0.1:8080/categories \
  -H "Content-Type: application/json" \
  -d '{"user_id":1,"name":"Groceries"}'

curl -s 'http://127.0.0.1:8080/categories?page=1&page_size=20'
```

### Create a transaction with a category split and fetch
```bash
curl -X POST http://127.0.0.1:8080/transactions \
  -H "Content-Type: application/json" \
  -d '{"account_id":1,"amount":50,"transaction_type":"expense","description":"Food","categories":[{"category_id":1,"amount":50}]}'

curl -s http://127.0.0.1:8080/transactions
```

### Recurring transactions
- Recurring: `POST /recurring_transactions`, `GET /recurring_transactions`, `PUT /recurring_transactions/{id}`, `DELETE /recurring_transactions/{id}`.

### Exchange rates - Create and query
```bash
# Create exchange rate manually
curl -X POST http://127.0.0.1:8080/exchange-rates \
  -H "Content-Type: application/json" \
  -d '{"from_currency":"USD","to_currency":"Euro (EUR)","rate":0.92,"source":"manual"}'

# List all exchange rates
curl -s 'http://127.0.0.1:8080/exchange-rates?page=1&page_size=20'

# Filter by source currency
curl -s 'http://127.0.0.1:8080/exchange-rates?from_currency=USD'

# Filter by target currency (partial match)
curl -s 'http://127.0.0.1:8080/exchange-rates?to_currency=EUR'

# Filter by date
curl -s 'http://127.0.0.1:8080/exchange-rates?date=2024-12-06'

# Combine filters
curl -s 'http://127.0.0.1:8080/exchange-rates?from_currency=USD&source=scraper&date=2025-12-05'
```

### Exchange rates - Get latest rates
```bash
# Get latest rates for USD
curl -s http://127.0.0.1:8080/exchange-rates/latest/USD

# Get latest rates for CAD
curl -s http://127.0.0.1:8080/exchange-rates/latest/CAD
```

### Exchange rates - Currency conversion
```bash
# Convert 100 USD to EUR
curl -s 'http://127.0.0.1:8080/exchange-rates/convert?from=USD&to=EUR&amount=100'

# Convert 500 CAD to USD
curl -s 'http://127.0.0.1:8080/exchange-rates/convert?from=CAD&to=USD&amount=500'
```

### Exchange rates - Update and delete
```bash
# Update exchange rate
curl -X PUT http://127.0.0.1:8080/exchange-rates/1 \
  -H "Content-Type: application/json" \
  -d '{"rate":0.93,"source":"api"}'

# Delete single exchange rate
curl -X DELETE http://127.0.0.1:8080/exchange-rates/1

# Bulk delete by source currency
curl -X DELETE 'http://127.0.0.1:8080/exchange-rates/bulk?from_currency=USD'

# Bulk delete by date
curl -X DELETE 'http://127.0.0.1:8080/exchange-rates/bulk?date=2024-12-06'

# Bulk delete with combined filters
curl -X DELETE 'http://127.0.0.1:8080/exchange-rates/bulk?from_currency=USD&date=2024-12-06&source=scraper'
```


### Update or delete resources
- Users: `PUT /users/{id}`, `DELETE /users/{id}`
- Accounts: `PUT /accounts/{id}`, `DELETE /accounts/{id}`
- Categories: `PUT /categories/{id}`, `DELETE /categories/{id}`
- Transactions: `PUT /transactions/{id}`, `DELETE /transactions/{id}`
- Recurring: `PUT /recurring_transactions/{id}`, `DELETE /recurring_transactions/{id}`
- Exchange rates: `PUT /exchange-rates/{id}`, `DELETE /exchange-rates/{id}`


## Request/Response Basics
- Content type: JSON bodies and JSON responses.
- Envelope: `{ "success": true/false, "data": ..., "message": ... }`.
- Pagination: `page`, `page_size` query params (defaults 1, 20).
- Timestamps: UTC ISO-8601.

## Expected Errors
- 400: validation failures (e.g., duplicate username/email, invalid enum, category split mismatch).
- 404: resource not found by id.
- 500: database or server errors (check server logs).

## Handy Dev Commands
- `cargo run serve` — start API.
- `cargo run db_seed` — seed sample data.
- `cargo run scrape_rates` - scrape foreign exchange rates for default currencies: CAD, USD, GBP, EUR.  
- `cargo run scrape_rates CAD` - scrape foreign exchange rates for CAD only.  
- `cargo run db_clear` — clear all data (prompts).
- `cargo run db_reseed` — clear then reseed.
- `cargo run tui` — launch the terminal UI.


## API Endpoints Summary

### Users
- `GET /users` - List users (paginated)
- `GET /users/{id}` - Get user by ID
- `POST /users` - Create user
- `PUT /users/{id}` - Update user
- `DELETE /users/{id}` - Delete user

### Accounts
- `GET /accounts` - List accounts (paginated)
- `GET /accounts/{id}` - Get account by ID
- `POST /accounts` - Create account
- `PUT /accounts/{id}` - Update account
- `DELETE /accounts/{id}` - Delete account

### Categories
- `GET /categories` - List categories (paginated)
- `GET /categories/{id}` - Get category by ID
- `POST /categories` - Create category
- `PUT /categories/{id}` - Update category
- `DELETE /categories/{id}` - Delete category

### Transactions
- `GET /transactions` - List transactions with filters (account_id, transaction_type)
- `GET /transactions/{id}` - Get transaction with categories
- `POST /transactions` - Create transaction (updates account balance)
- `PUT /transactions/{id}` - Update transaction
- `DELETE /transactions/{id}` - Delete transaction (reverses balance change)

### Recurring Transactions
- `GET /recurring_transactions` - List recurring transactions
- `GET /recurring_transactions/{id}` - Get recurring transaction by ID
- `POST /recurring_transactions` - Create recurring transaction
- `PUT /recurring_transactions/{id}` - Update recurring transaction
- `DELETE /recurring_transactions/{id}` - Delete recurring transaction

### Exchange Rates
- `GET /exchange-rates` - List exchange rates with filters (from_currency, to_currency, source, date)
- `GET /exchange-rates/{id}` - Get exchange rate by ID
- `GET /exchange-rates/latest/{from_currency}` - Get latest rates for a currency
- `GET /exchange-rates/convert` - Convert currency (params: from, to, amount)
- `POST /exchange-rates` - Create exchange rate
- `PUT /exchange-rates/{id}` - Update exchange rate
- `DELETE /exchange-rates/{id}` - Delete exchange rate
- `DELETE /exchange-rates/bulk` - Bulk delete (params: from_currency, date, source)

