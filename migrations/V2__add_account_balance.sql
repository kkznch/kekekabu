CREATE TABLE IF NOT EXISTS account_balance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cash_available TEXT NOT NULL,
    synced_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_account_balance_synced_at
    ON account_balance(synced_at DESC);
