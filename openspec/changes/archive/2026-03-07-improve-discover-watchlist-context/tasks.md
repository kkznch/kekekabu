## 1. Database

- [x] 1.1 `schema.rs` に `watchlist_events` テーブル定義を追加（id, ticker, action, reason, discovered_at）
- [x] 1.2 `ALL_SCHEMAS` に `CREATE_WATCHLIST_EVENTS_TABLE` を追加
- [x] 1.3 `db/mod.rs` に `save_watchlist_event(conn, ticker, action, reason)` 関数を追加

## 2. Discover プロンプト改修

- [x] 2.1 `build_discover_prompt` に `watchlist_context: Option<&str>` パラメータを追加し、現在のウォッチリストをプロンプトに含める
- [x] 2.2 LLM レスポンススキーマを `{ candidates: [...] }` から `{ keep: [...], add: [...], remove: [...] }` に変更（`DiscoverResponse`, `DiscoverCandidate` 構造体の更新）
- [x] 2.3 `run()` 関数で現在の watchlist を取得してプロンプトに渡すように変更

## 3. 差分管理ロジック改修

- [x] 3.1 LLM の keep/add/remove レスポンスに基づく差分管理に書き替え（現在の機械的な差分を置換）
- [x] 3.2 keep/add/remove いずれにも含まれない銘柄は変更なし（watchlist に残す）として扱う
- [x] 3.3 各アクション（add/remove/keep）の実行時に `save_watchlist_event` を呼び出す

## 4. テスト

- [x] 4.1 `parse_discover_response` のテストを新スキーマ（keep/add/remove）に更新
- [x] 4.2 `save_watchlist_event` と `watchlist_events` テーブルのテストを追加
