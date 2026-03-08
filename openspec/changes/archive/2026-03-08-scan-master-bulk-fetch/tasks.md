## 1. J-Quants API 拡張

- [x] 1.1 `jquants.rs` に `get_all_stock_info()` 関数を追加（`equities/master` パラメータなし呼び出し、`Vec<ListedInfo>` を返す）
- [x] 1.2 `get_all_stock_info()` のレスポンスパースが既存の `ListedInfo` 構造体で動作することを確認

## 2. DB 層

- [x] 2.1 `db/mod.rs` に `save_stocks_bulk()` 関数を追加（`Vec<ListedInfo>` を受け取り、トランザクション内で全件 UPSERT）
- [x] 2.2 `db/mod.rs` に `has_any_stocks()` 関数を追加（stocks テーブルにレコードが1件以上あるかを返す）

## 3. scan コマンド改修

- [x] 3.1 `main.rs` の Scan コマンドに `--refresh-master` フラグを追加
- [x] 3.2 `scan.rs` の `run()` に `refresh_master` パラメータを追加
- [x] 3.3 `--refresh-master` 指定時: `get_all_stock_info()` → `save_stocks_bulk()` を実行してから scan 処理に入る
- [x] 3.4 `--refresh-master` なし + stocks 空: エラーメッセージ表示で中断
- [x] 3.5 ループから `get_stock_info()` 呼び出しと前後の sleep(1s) を削除
- [x] 3.6 銘柄間 sleep を 1s → 0.3s に短縮
- [x] 3.7 watchlist の銘柄が stocks テーブルに未登録の場合はスキップ + 警告ログ

## 4. テスト

- [x] 4.1 `save_stocks_bulk()` のテスト（空、複数件、UPSERT 上書き確認）
- [x] 4.2 `has_any_stocks()` のテスト（空テーブル、データあり）

## 5. ドキュメント更新

- [x] 5.1 CLAUDE.md の Commands セクションに `--refresh-master` を追記
- [x] 5.2 README.md の cron セクションに週次 `--refresh-master` の記載を追加
