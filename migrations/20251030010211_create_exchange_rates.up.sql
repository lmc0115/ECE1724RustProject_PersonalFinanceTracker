CREATE TABLE IF NOT EXISTS exchange_rates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    from_currency TEXT NOT NULL,
    to_currency TEXT NOT NULL,
    
    -- Exchange rate: how much of to_currency equals 1 unit of from_currency
    -- Example: USD to EUR rate of 0.85 means 1 USD = 0.85 EUR
    rate REAL NOT NULL,
    rate_date TIMESTAMP NOT NULL,
    source TEXT NOT NULL CHECK(source IN ('api', 'bank', 'manual', 'scraper')),
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(from_currency, to_currency, rate_date)
);

-- Create trigger to automatically update updated_at on row modification
CREATE TRIGGER IF NOT EXISTS update_exchange_rates_updated_at
    AFTER UPDATE ON exchange_rates
    FOR EACH ROW
BEGIN
    UPDATE exchange_rates 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = NEW.id;
END;

CREATE INDEX IF NOT EXISTS idx_exchange_rates_currencies 
    ON exchange_rates(from_currency, to_currency);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_date 
    ON exchange_rates(rate_date);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_currencies_date 
    ON exchange_rates(from_currency, to_currency, rate_date DESC);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_source 
    ON exchange_rates(source);