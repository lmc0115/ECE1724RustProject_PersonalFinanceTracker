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
- Exchange rates: unique `(from_currency, to_currency, rate_date)`; `source` in `api`, `bank`, `manual`.

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

### Update or delete resources
- Users: `PUT /users/{id}`, `DELETE /users/{id}`
- Accounts: `PUT /accounts/{id}`, `DELETE /accounts/{id}`
- Categories: `PUT /categories/{id}`, `DELETE /categories/{id}`
- Transactions: `PUT /transactions/{id}`, `DELETE /transactions/{id}`

### Recurring transactions and exchange rates
- Recurring: `POST /recurring_transactions`, `GET /recurring_transactions`, `PUT /recurring_transactions/{id}`, `DELETE /recurring_transactions/{id}`.
- Exchange rates: `POST /exchange_rates`, `GET /exchange_rates`, `PUT /exchange_rates/{id}`, `DELETE /exchange_rates/{id}`.

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
- `cargo run db_clear` — clear all data (prompts).
- `cargo run db_reseed` — clear then reseed.
- `cargo run tui` — launch the terminal UI.
