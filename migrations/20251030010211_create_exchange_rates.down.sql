-- Drop trigger first
DROP TRIGGER IF EXISTS update_exchange_rates_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_exchange_rates_source;
DROP INDEX IF EXISTS idx_exchange_rates_currencies_date;
DROP INDEX IF EXISTS idx_exchange_rates_date;
DROP INDEX IF EXISTS idx_exchange_rates_currencies;

-- Drop Exchange_Rates table
DROP TABLE IF EXISTS exchange_rates;

