-- Drop indexes
DROP INDEX IF EXISTS idx_transaction_categories_category_amount;
DROP INDEX IF EXISTS idx_transaction_categories_category_id;
DROP INDEX IF EXISTS idx_transaction_categories_transaction_id;

-- Drop Transaction_Categories junction table
DROP TABLE IF EXISTS transaction_categories;