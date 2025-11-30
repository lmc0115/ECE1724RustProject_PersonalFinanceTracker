-- Drop trigger first
DROP TRIGGER IF EXISTS update_categories_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_categories_user_id;

-- Drop Categories table
DROP TABLE IF EXISTS categories;