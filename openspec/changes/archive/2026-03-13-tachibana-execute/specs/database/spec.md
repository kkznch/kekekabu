## MODIFIED Requirements

### Requirement: SQLite を使った7テーブル構成のデータベース
システムは SHALL SQLite（tokio-rusqlite, bundled）を使用し、stocks, prices, watchlist, evaluations, fetch_results, portfolio_positions, trades, watchlist_events, llm_logs, orders の10テーブルを管理する。

#### Scenario: データベース初期化
- **WHEN** アプリケーション起動時（init 以外の任意のコマンド）
- **THEN** ~/.config/kabu/kekekabu.db にデータベースファイルを作成し、10テーブルすべてが存在することを保証する

## ADDED Requirements

### Requirement: orders テーブル
システムは SHALL 注文ライフサイクルを追跡する orders テーブルを管理する。stock_id, side, order_type, price, quantity, status, tachibana_order_id, request_id (UNIQUE), filled_price, filled_quantity, filled_at, evaluation_id を保持する。

#### Scenario: 冪等な注文挿入
- **WHEN** 同一 request_id の注文が再度挿入された場合
- **THEN** 重複を無視する（INSERT OR IGNORE）
