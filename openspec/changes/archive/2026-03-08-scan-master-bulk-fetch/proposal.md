## Why

scan コマンドが1銘柄ずつ stock_info API を呼んでおり、銘柄数に比例して sleep + API 往復で時間がかかる。全上場銘柄マスターを一括取得して DB にキャッシュすれば、scan から stock_info API 呼び出しと sleep を削除でき、実行時間を約半分に短縮できる。

## What Changes

- J-Quants `equities/master`（パラメータなし）で全上場銘柄を一括取得する関数を追加
- `scan --refresh-master` フラグを追加。指定時に一括取得して stocks テーブルに UPSERT
- scan ループから `get_stock_info()` 呼び出しと前後の sleep(1s) を削除
- scan ループの銘柄間 sleep を 1s → 0.3s に短縮（429 リトライが既にあるため）
- stocks テーブルが空で `--refresh-master` なしの場合はエラーで案内

## Capabilities

### New Capabilities

- `stock-master`: 全上場銘柄マスターの一括取得・キャッシュ管理

### Modified Capabilities

- `data-pipeline`: scan コマンドから stock_info 個別取得を削除し、DB キャッシュ参照に変更。sleep 間隔を短縮

## Impact

- `src/jquants.rs`: `get_all_stock_info()` 関数追加
- `src/cmd/scan.rs`: `--refresh-master` フラグ追加、ループ構造変更、sleep 短縮
- `src/main.rs`: scan コマンドに `--refresh-master` オプション追加
- `src/db/mod.rs`: 一括 UPSERT 関数追加（既存 `save_stock()` のバッチ版）
- CLAUDE.md / README.md: cron セクションに週次 `--refresh-master` を追記
