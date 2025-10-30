CREATE TABLE IF NOT EXISTS transaction_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    transaction_id INTEGER NOT NULL,
        category_id INTEGER NOT NULL,
    
    -- Amount allocated to this category (for split transactions)
    -- Example: $100 grocery bill split into $60 food + $40 household items
    amount REAL NOT NULL,
    
    FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE,
    
    UNIQUE(transaction_id, category_id)
);

CREATE INDEX IF NOT EXISTS idx_transaction_categories_transaction_id 
    ON transaction_categories(transaction_id);

CREATE INDEX IF NOT EXISTS idx_transaction_categories_category_id 
    ON transaction_categories(category_id);

CREATE INDEX IF NOT EXISTS idx_transaction_categories_category_amount 
    ON transaction_categories(category_id, amount);