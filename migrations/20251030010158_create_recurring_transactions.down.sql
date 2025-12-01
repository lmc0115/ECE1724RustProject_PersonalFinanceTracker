-- Drop trigger first
DROP TRIGGER IF EXISTS update_recurring_transactions_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_recurring_transactions_category_id;
DROP INDEX IF EXISTS idx_recurring_transactions_active_next;
DROP INDEX IF EXISTS idx_recurring_transactions_is_active;
DROP INDEX IF EXISTS idx_recurring_transactions_next_occurrence;
DROP INDEX IF EXISTS idx_recurring_transactions_account_id;

-- Drop Recurring_Transactions table
DROP TABLE IF EXISTS recurring_transactions;