## MODIFIED Requirements

### Requirement: SQLite を使った10テーブル構成のデータベース
システムは SHALL SQLite（tokio-rusqlite, bundled）を使用し、stocks, prices, watchlist, evaluations, fetch_results, portfolio_positions, trades, watchlist_events, llm_logs, orders の10テーブルを管理する。DB アクセスは `DbClient` trait を通じて行い、`SqliteClient` が実装を提供する。スキーマは refinery マイグレーション（`migrations/` ディレクトリ）で管理する。`db_path()` 関数を `pub` で公開し、DB ファイルパスを外部から取得可能にする。`SqliteClient` は `migration_status()` メソッドで適用済みマイグレーション情報（`MigrationInfo`）を提供する。

#### Scenario: データベース初期化（明示的）
- **WHEN** `kabu db migrate` を実行した場合
- **THEN** `SqliteClient::open_or_create()` により DB ファイルが存在しなければ親ディレクトリごと作成し、refinery マイグレーションを適用して全テーブルを作成する

#### Scenario: DB 不在時のエラー
- **WHEN** DB ファイルが存在しない状態で `SqliteClient::open()` を呼び出した場合
- **THEN** `Database not found at <path>` エラーを返し、`kabu db migrate` の実行を案内する

#### Scenario: 既存 DB のマイグレーション適用
- **WHEN** DB ファイルが存在する状態で `SqliteClient::open()` を呼び出した場合
- **THEN** refinery マイグレーションを適用し（未適用分のみ）、WAL モードと busy_timeout=5000ms を設定する

#### Scenario: テスト用インメモリ DB
- **WHEN** `SqliteClient::open_in_memory()` を呼び出した場合
- **THEN** インメモリ SQLite DB を作成し、`refinery::embed_migrations!` で本番同様に全マイグレーションを適用する

#### Scenario: DB パスの取得
- **WHEN** `db::db_path()` を呼び出した場合
- **THEN** `~/.config/kabu/kekekabu.db` のパスを返す

#### Scenario: マイグレーション状態の取得
- **WHEN** `db.migration_status()` を呼び出した場合
- **THEN** `refinery_schema_history` テーブルから適用済みマイグレーションの version, name, applied_on を `Vec<MigrationInfo>` として返す。テーブルが存在しない場合は空の Vec を返す
