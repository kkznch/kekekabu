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
    decision TEXT NOT NULL CHECK(decision IN ('Buy', 'Hold', 'Sell', 'Avoid')),
    score INTEGER NOT NULL CHECK(score BETWEEN 0 AND 100),
    rationale TEXT NOT NULL,
    ta_summary TEXT,
    spec_hash TEXT,
    llm_backend TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const CREATE_FETCH_RESULTS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS fetch_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id),
    source TEXT NOT NULL,
    category TEXT NOT NULL CHECK(category IN ('news', 'disclosure', 'sentiment', 'competitor', 'other')),
    title TEXT NOT NULL,
    url TEXT,
    body TEXT,
    published_at TEXT,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(stock_id, url)
);
";

pub const CREATE_PORTFOLIO_POSITIONS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS portfolio_positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id) UNIQUE,
    quantity TEXT NOT NULL,
    avg_cost TEXT NOT NULL,
    total_invested TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    opened_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const CREATE_TRADES_TABLE: &str = "
CREATE TABLE IF NOT EXISTS trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id),
    side TEXT NOT NULL CHECK(side IN ('buy', 'sell')),
    date TEXT NOT NULL,
    price TEXT NOT NULL,
    quantity TEXT NOT NULL,
    pnl TEXT,
    strategy TEXT,
    order_type TEXT,
    stop_loss_price TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const CREATE_WATCHLIST_EVENTS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS watchlist_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker TEXT NOT NULL,
    action TEXT NOT NULL CHECK(action IN ('add', 'remove', 'keep', 'auto-removed-on-sell')),
    reason TEXT,
    discovered_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const CREATE_LLM_LOGS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS llm_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command TEXT NOT NULL,
    ticker TEXT,
    backend TEXT NOT NULL,
    model TEXT,
    temperature REAL,
    prompt TEXT NOT NULL,
    response TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub const ALL_SCHEMAS: &[&str] = &[
    CREATE_STOCKS_TABLE,
    CREATE_PRICES_TABLE,
    CREATE_WATCHLIST_TABLE,
    CREATE_EVALUATIONS_TABLE,
    CREATE_FETCH_RESULTS_TABLE,
    CREATE_PORTFOLIO_POSITIONS_TABLE,
    CREATE_TRADES_TABLE,
    CREATE_WATCHLIST_EVENTS_TABLE,
    CREATE_LLM_LOGS_TABLE,
];
