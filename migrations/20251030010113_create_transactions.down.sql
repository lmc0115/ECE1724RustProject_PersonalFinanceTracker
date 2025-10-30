-- Drop trigger first
DROP TRIGGER IF EXISTS update_transactions_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_transactions_account_type;
DROP INDEX IF EXISTS idx_transactions_account_date;
DROP INDEX IF EXISTS idx_transactions_type;
DROP INDEX IF EXISTS idx_transactions_date;
DROP INDEX IF EXISTS idx_transactions_account_id;

-- Drop Transactions table
DROP TABLE IF EXISTS transactions;