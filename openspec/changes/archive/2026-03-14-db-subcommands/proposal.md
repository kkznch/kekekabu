## Why

CLI からデータベースの管理操作（マイグレーション実行・状態確認・リセット）を行う手段がなく、開発時やトラブルシューティング時にデータベースの状態を把握・操作するのが困難。`kabu db` サブコマンドを追加し、DB 管理を CLI から直接行えるようにする。

## What Changes

- `kabu db migrate` — マイグレーション実行と適用状況の表示
- `kabu db status` — DB パス・サイズ・マイグレーション履歴の表示
- `kabu db reset` — ランダム確認コードによる対話式の DB 削除（`--force` でスキップ可能）
- `MigrationInfo` 構造体と `migration_status()` メソッドを `SqliteClient` に追加
- `db_path()` を `pub` に公開

## Capabilities

### New Capabilities
- `db-management`: CLI からのデータベース管理操作（マイグレーション実行・状態確認・リセット）

### Modified Capabilities
- `database`: `db_path()` の公開と `MigrationInfo`/`migration_status()` の追加

## Impact

- `src/cmd/db.rs` — 新規ファイル（migrate, status, reset ハンドラ）
- `src/cmd/mod.rs` — `pub mod db` 追加
- `src/db/mod.rs` — `db_path()` 公開、`MigrationInfo` 構造体、`migration_status()` メソッド追加
- `src/main.rs` — `DbCommand` enum、`Db(DbCommand)` バリアント、ルーティング追加
- `src/output.rs` — `MigrationInfo` の `HumanDisplay` 実装
- `Cargo.toml` — `rand` クレート追加（リセット確認コード生成用）
