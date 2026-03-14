## MODIFIED Requirements

### Requirement: SQLite を使った10テーブル構成のデータベース
システムは SHALL SQLite（tokio-rusqlite, bundled）を使用し、stocks, prices, watchlist, evaluations, fetch_results, portfolio_positions, trades, watchlist_events, llm_logs, orders の10テーブルを管理する。DB アクセスは `DbClient` trait を通じて行い、`SqliteClient` が実装を提供する。スキーマは refinery マイグレーション（`migrations/` ディレクトリ）で管理する。

#### Scenario: データベース初期化
- **WHEN** アプリケーション起動時（`SqliteClient::open()` 呼び出し時）
- **THEN** ~/.config/kabu/kekekabu.db にデータベースファイルを作成し、refinery マイグレーションを自動適用して10テーブルすべてが存在することを保証する。WAL モードと busy_timeout=5000ms を設定する

#### Scenario: テスト用インメモリ DB
- **WHEN** `SqliteClient::open_in_memory()` を呼び出した場合
- **THEN** インメモリ SQLite DB を作成し、V1 マイグレーション SQL を直接実行してテスト用の全テーブルを作成する

## ADDED Requirements

### Requirement: DbClient trait による DB アクセス抽象化
システムは SHALL `DbClient` trait を定義し、全 DB 操作（stocks, prices, watchlist, evaluations, fetch_results, portfolio, trades, llm_logs, orders）を async メソッドとして提供する。全コマンドハンドラは `&dyn DbClient` を受け取る。

#### Scenario: コマンドハンドラへの DI
- **WHEN** コマンドハンドラ（discover, scan, fetch, eval, execute, report, show, workflow）が DB 操作を行う場合
- **THEN** `&dyn DbClient` 経由でメソッドを呼び出し、具体的な DB 実装に依存しない

#### Scenario: SqliteClient が DbClient を実装
- **WHEN** `SqliteClient` が生成された場合
- **THEN** `DbClient` trait の全メソッド（36メソッド）を実装し、tokio-rusqlite の `conn.call()` で同期 SQLite 操作をラップする

### Requirement: refinery によるスキーママイグレーション
システムは SHALL refinery クレートを使用してデータベーススキーマのバージョン管理を行う。マイグレーションファイルは `migrations/` ディレクトリに `V{番号}__{説明}.sql` の命名規則で配置する。

#### Scenario: 初回マイグレーション適用
- **WHEN** 新規データベースに対して `SqliteClient::open()` を呼び出した場合
- **THEN** `migrations/V1__initial_schema.sql` を適用し、全テーブルを作成する

#### Scenario: 適用済みマイグレーションのスキップ
- **WHEN** 既にマイグレーション適用済みのデータベースに対して `SqliteClient::open()` を呼び出した場合
- **THEN** `refinery_schema_history` テーブルを参照し、適用済みのマイグレーションをスキップする
