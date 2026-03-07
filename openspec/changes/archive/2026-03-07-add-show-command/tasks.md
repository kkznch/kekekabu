## 1. DB クエリ追加

- [x] 1.1 `db/mod.rs` に `list_watchlist_events(conn, ticker: Option<&str>)` 関数を追加
- [x] 1.2 `db/mod.rs` に `list_stocks(conn)` 関数を追加
- [x] 1.3 `db/mod.rs` に `table_stats(conn)` 関数を追加（全テーブルのレコード数）

## 2. show コマンド実装

- [x] 2.1 `src/cmd/show.rs` を作成し、ShowCommand enum と各サブコマンドのハンドラを実装
- [x] 2.2 `src/cmd/mod.rs` に `pub mod show;` を追加
- [x] 2.3 `src/main.rs` に `Show` サブコマンドを追加し、ハンドラを接続

## 3. 既存コマンド統合・廃止

- [x] 3.1 `src/main.rs` から `History` サブコマンドを削除
- [x] 3.2 `src/main.rs` の `Discover` から `--list` フラグを削除し、`cmd/discover.rs` の `list()` 関数を削除
- [x] 3.3 show watchlist / show evaluations で既存の DB 関数を再利用

## 4. 出力フォーマット

- [x] 4.1 `src/output.rs` に watchlist_events, stocks, table_stats 用の HumanDisplay を実装

## 5. ドキュメント更新

- [x] 5.1 `README.md` のコマンド一覧を更新（show 追加、history / discover --list 削除）
- [x] 5.2 `CLAUDE.md` のコマンド一覧を更新

## 6. テスト

- [x] 6.1 `tests/db_test.rs` に watchlist_events 取得、stocks 一覧、table_stats のテストを追加
- [x] 6.2 全テスト通過を確認
