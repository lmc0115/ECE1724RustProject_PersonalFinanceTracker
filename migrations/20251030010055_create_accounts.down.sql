-- Drop trigger first
DROP TRIGGER IF EXISTS update_accounts_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_accounts_currency;
DROP INDEX IF EXISTS idx_accounts_type;
DROP INDEX IF EXISTS idx_accounts_user_id;

-- Drop Accounts table
DROP TABLE IF EXISTS accounts;