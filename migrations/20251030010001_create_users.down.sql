-- Drop trigger first
DROP TRIGGER IF EXISTS update_users_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_users_username;
DROP INDEX IF EXISTS idx_users_email;

-- Drop Users table
DROP TABLE IF EXISTS users;