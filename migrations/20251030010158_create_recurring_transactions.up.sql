CREATE TABLE IF NOT EXISTS recurring_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    account_id INTEGER NOT NULL,
    category_id INTEGER,
    
    -- Transaction amount (positive for income, negative for expenses)
    amount REAL NOT NULL,
    
    -- Transaction type: income or expense (transfers handled separately)
    transaction_type TEXT NOT NULL CHECK(transaction_type IN ('income', 'expense')),
    description TEXT,
    
    frequency TEXT NOT NULL CHECK(frequency IN ('daily', 'weekly', 'monthly', 'yearly')),
    
    start_date TIMESTAMP NOT NULL,
    end_date TIMESTAMP,
    next_occurrence TIMESTAMP NOT NULL,
    
    is_active BOOLEAN NOT NULL DEFAULT 1,
    
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL,
    
    CHECK (end_date IS NULL OR end_date > start_date)
);

-- Create trigger to automatically update updated_at on row modification
CREATE TRIGGER IF NOT EXISTS update_recurring_transactions_updated_at
    AFTER UPDATE ON recurring_transactions
    FOR EACH ROW
BEGIN
    UPDATE recurring_transactions 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = NEW.id;
END;

CREATE INDEX IF NOT EXISTS idx_recurring_transactions_account_id 
    ON recurring_transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_next_occurrence 
    ON recurring_transactions(next_occurrence);
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_is_active 
    ON recurring_transactions(is_active);
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_active_next 
    ON recurring_transactions(is_active, next_occurrence);
CREATE INDEX IF NOT EXISTS idx_recurring_transactions_category_id 
    ON recurring_transactions(category_id);