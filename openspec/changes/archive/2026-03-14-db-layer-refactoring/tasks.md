## 1. refinery マイグレーション導入

- [x] 1.1 Cargo.toml に refinery + refinery-core 依存を追加
- [x] 1.2 src/db/schema.rs を migrations/V1__initial_schema.sql に移行
- [x] 1.3 SqliteClient::open() で refinery マイグレーションを自動適用するように実装

## 2. DbClient trait + SqliteClient 実装

- [x] 2.1 DbClient trait を定義（36 async メソッド: stocks, prices, watchlist, evaluations, fetch_results, portfolio, trades, llm_logs, orders）
- [x] 2.2 SqliteClient 構造体を作成し DbClient trait を実装
- [x] 2.3 FillParams 構造体を db/mod.rs に移動し update_order_and_record_fill を trait メソッドとして実装
- [x] 2.4 portfolio の async 関数（list_positions, portfolio_summary, trade_history）を DbClient に移動
- [x] 2.5 portfolio_buy/portfolio_sell を SqliteClient のメソッドとして実装（buy_sync/sell_sync を pub(crate) でラップ）
- [x] 2.6 SqliteClient::open_in_memory() をテスト用に実装

## 3. コマンドハンドラの DI 移行

- [x] 3.1 cmd/discover.rs を &dyn DbClient に移行
- [x] 3.2 cmd/scan.rs を &dyn DbClient に移行
- [x] 3.3 cmd/fetch.rs を &dyn DbClient に移行
- [x] 3.4 cmd/eval.rs を &dyn DbClient に移行
- [x] 3.5 cmd/execute.rs を &dyn DbClient に移行（FillParams の import 変更含む）
- [x] 3.6 cmd/report.rs を &dyn DbClient に移行
- [x] 3.7 cmd/show.rs の全10関数を &dyn DbClient に移行
- [x] 3.8 cmd/workflow.rs の全関数を &dyn DbClient に移行
- [x] 3.9 circuit_breaker.rs を &dyn DbClient に移行

## 4. main.rs とテストの更新

- [x] 4.1 main.rs の mod 宣言を廃止し use kekekabu::* に変更
- [x] 4.2 tests/db_test.rs を SqliteClient::open_in_memory() ベースに移行
- [x] 4.3 tests/portfolio_test.rs を db.portfolio_buy()/portfolio_sell() ベースに移行
- [x] 4.4 tests/scan_test.rs を SqliteClient ベースに移行

## 5. 検証

- [x] 5.1 cargo fmt -- --check が通ることを確認
- [x] 5.2 cargo clippy -- -D warnings が通ることを確認
- [x] 5.3 cargo test で全109テストが通ることを確認
