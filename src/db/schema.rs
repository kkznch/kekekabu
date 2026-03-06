pub const CREATE_STOCKS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS stocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    market TEXT NOT NULL DEFAULT 'jp' CHECK(market = 'jp'),
    sector TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const CREATE_PRICES_TABLE: &str = "
CREATE TABLE IF NOT EXISTS prices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id),
    date TEXT NOT NULL,
    open TEXT NOT NULL,
    high TEXT NOT NULL,
    low TEXT NOT NULL,
    close TEXT NOT NULL,
    volume INTEGER NOT NULL,
    adjusted_close TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(stock_id, date)
);
";

pub const CREATE_WATCHLIST_TABLE: &str = "
CREATE TABLE IF NOT EXISTS watchlist (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id) UNIQUE,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    notes TEXT
);
";

pub const CREATE_EVALUATIONS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS evaluations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id),
    evaluated_at TEXT NOT NULL DEFAULT (datetime('now')),
    decision TEXT NOT NULL CHECK(decision IN ('Buy', 'Hold', 'Avoid')),
    score INTEGER NOT NULL CHECK(score BETWEEN 0 AND 100),
    rationale TEXT NOT NULL,
    ta_summary TEXT,
    spec_hash TEXT,
    llm_backend TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const ALL_SCHEMAS: &[&str] = &[
    CREATE_STOCKS_TABLE,
    CREATE_PRICES_TABLE,
    CREATE_WATCHLIST_TABLE,
    CREATE_EVALUATIONS_TABLE,
];
