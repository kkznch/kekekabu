## MODIFIED Requirements

### Requirement: SQLite を使った10テーブル構成のデータベース
システムは SHALL SQLite（tokio-rusqlite, bundled）を使用し、stocks, prices, watchlist, evaluations, fetch_results, portfolio_positions, trades, watchlist_events, llm_logs, orders の10テーブルを管理する。

#### Scenario: データベース初期化
- **WHEN** アプリケーション起動時（init 以外の任意のコマンド）
- **THEN** ~/.config/kabu/kekekabu.db にデータベースファイルを作成し、10テーブルすべてが存在することを保証する。WAL モードと busy_timeout=5000ms を設定する

### Requirement: prices テーブル
システムは SHALL 日足 OHLCV データを (ticker, date) をユニークキーとして保存する。

#### Scenario: 冪等な価格データ挿入
- **WHEN** 同一 ticker・同一日付の価格データが再度挿入された場合
- **THEN** 既存レコードを最新データで更新する（ON CONFLICT DO UPDATE）

## ADDED Requirements

### Requirement: スキーマバージョン管理
システムは SHALL `PRAGMA user_version` によりスキーマバージョンを管理し、起動時にマイグレーションを実行する。

#### Scenario: 初回起動時のバージョン設定
- **WHEN** user_version が 0（未設定）のデータベースで起動した場合
- **THEN** テーブル作成後に user_version を現在の SCHEMA_VERSION に設定する

#### Scenario: マイグレーションの適用
- **WHEN** user_version が SCHEMA_VERSION より小さいデータベースで起動した場合
- **THEN** バージョン差分に対応する ALTER TABLE 等のマイグレーションを順次適用し、user_version を更新する
