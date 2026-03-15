## Purpose

SQLite による永続化層。10 テーブル構成で冪等書き込みを保証し、WAL モード・busy_timeout で並行アクセスに対応する。rust_decimal による金額精度を維持する全コマンドのデータ基盤。DbClient trait による DI パターンで抽象化し、refinery によるスキーママイグレーション管理を行う。

## Requirements

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

### Requirement: stocks テーブル
システムは SHALL 銘柄マスタデータ（ticker, name, sector）を ticker をユニークキーとして保存する。

#### Scenario: 銘柄の upsert
- **WHEN** 同一 ticker で name/sector が更新された銘柄データが保存された場合
- **THEN** 既存レコードを更新する（ON CONFLICT UPDATE）

### Requirement: prices テーブル
システムは SHALL 日足 OHLCV データを (ticker, date) をユニークキーとして保存する。

#### Scenario: 冪等な価格データ挿入
- **WHEN** 同一 ticker・同一日付の価格データが再度挿入された場合
- **THEN** 既存レコードを最新データで更新する（ON CONFLICT DO UPDATE）

### Requirement: 金額を TEXT 型で保存
システムは SHALL すべての金額を SQLite 上で TEXT 型として保存し、rust_decimal::Decimal で精度を保証する。

#### Scenario: Decimal の精度保持
- **WHEN** 2345.50 という価格を保存して読み戻した場合
- **THEN** 浮動小数点の丸め誤差なく正確な値が復元される

### Requirement: evaluations テーブルに spec_hash を記録
システムは SHALL 評価結果を、使用した投資 Spec の SHA256 ハッシュとともに保存する。

#### Scenario: Spec 追跡付き評価保存
- **WHEN** 評価が保存される場合
- **THEN** 評価に使用した Spec バージョンに紐づく spec_hash フィールドが含まれる

### Requirement: fetch_results テーブル
システムは SHALL LLM が収集した情報（category, content, source）を ticker ごとに保存する。

#### Scenario: 1銘柄に対する複数カテゴリのデータ
- **WHEN** fetch がある銘柄のニュース、開示、センチメントを収集した場合
- **THEN** それぞれを適切な category で別行として保存する

### Requirement: ポートフォリオ関連テーブル
システムは SHALL portfolio_positions（保有ポジション・加重平均コスト）と trades（売買履歴・P&L）テーブルを使用する。

#### Scenario: ポジションのライフサイクル
- **WHEN** 買い → 一部売り → 全売りの一連の操作が行われた場合
- **THEN** portfolio_positions が quantity/avg_cost を追跡し、trades が各取引を P&L 付きで記録する

### Requirement: orders テーブル
システムは SHALL 注文ライフサイクルを追跡する orders テーブルを管理する。stock_id, side, order_type, price, quantity, status, tachibana_order_id, request_id (UNIQUE), filled_price, filled_quantity, filled_at, evaluation_id を保持する。

#### Scenario: 冪等な注文挿入
- **WHEN** 同一 request_id の注文が再度挿入された場合
- **THEN** 重複を無視する（INSERT OR IGNORE）

### Requirement: DbClient trait による DB アクセス抽象化
システムは SHALL `DbClient` trait を定義し、全 DB 操作（stocks, prices, watchlist, evaluations, fetch_results, portfolio, trades, llm_logs, orders）を async メソッドとして提供する。全コマンドハンドラは `&dyn DbClient` を受け取る。

#### Scenario: コマンドハンドラへの DI
- **WHEN** コマンドハンドラ（discover, scan, fetch, eval, execute, report, show, workflow）が DB 操作を行う場合
- **THEN** `&dyn DbClient` 経由でメソッドを呼び出し、具体的な DB 実装に依存しない

#### Scenario: SqliteClient が DbClient を実装
- **WHEN** `SqliteClient` が生成された場合
- **THEN** `DbClient` trait の全メソッドを実装し、tokio-rusqlite の `conn.call()` で同期 SQLite 操作をラップする

### Requirement: refinery によるスキーママイグレーション
システムは SHALL refinery クレートを使用してデータベーススキーマのバージョン管理を行う。マイグレーションファイルは `migrations/` ディレクトリに `V{番号}__{説明}.sql` の命名規則で配置する。

#### Scenario: 初回マイグレーション適用
- **WHEN** 新規データベースに対して `SqliteClient::open_or_create()` を呼び出した場合
- **THEN** `migrations/V1__initial_schema.sql` を適用し、全テーブルを作成する

#### Scenario: 適用済みマイグレーションのスキップ
- **WHEN** 既にマイグレーション適用済みのデータベースに対して `SqliteClient::open()` を呼び出した場合
- **THEN** `refinery_schema_history` テーブルを参照し、適用済みのマイグレーションをスキップする
