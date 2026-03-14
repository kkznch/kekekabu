## Why

DB 操作が自由関数（`db::save_stock(conn, ...)` 等）として実装されており、テスト時のモック差し替えが困難で、各コマンドが `Connection` を直接受け取る密結合な設計だった。また、スキーマ定義が `schema.rs` にハードコードされており、マイグレーション管理の仕組みがなかった。

## What Changes

- **BREAKING**: DB 操作の全自由関数を廃止し、`DbClient` trait（36 async メソッド）+ `SqliteClient` 実装に移行
- **BREAKING**: 全コマンドハンドラの引数を `&Connection` から `&dyn DbClient` に変更（DI パターン）
- `portfolio.rs` の async 関数群を `SqliteClient` のメソッドに移動、sync 関数を `pub(crate)` に変更
- `FillParams` 構造体を `execute.rs` から `db/mod.rs` に移動しトランザクション処理を一元化
- `schema.rs` を廃止し、refinery マイグレーション（`migrations/V1__initial_schema.sql`）に移行
- `main.rs` を `mod` 宣言方式からライブラリクレート参照方式（`use kekekabu::*`）に変更

## Capabilities

### New Capabilities

_(なし — 既存機能の内部リファクタリングのみ)_

### Modified Capabilities

- `database`: DB アクセス層を自由関数から DbClient trait + SqliteClient 実装に変更。refinery マイグレーション導入
- `portfolio`: async 関数を DbClient/SqliteClient に移動、sync 関数の可視性を pub(crate) に変更
- `trade-execution`: FillParams を db/mod.rs に移動、トランザクション処理を DbClient 経由に統一

## Impact

- **src/db/mod.rs**: 全面書き換え（DbClient trait 定義 + SqliteClient 実装、約 1400 行）
- **src/db/schema.rs**: 削除（migrations/ に移行）
- **src/portfolio.rs**: async 関数削除、sync 関数の可視性変更
- **src/cmd/*.rs**: 全 9 ファイルの関数シグネチャ変更（`&Connection` → `&dyn DbClient`）
- **src/circuit_breaker.rs**: 同上
- **src/main.rs**: `mod` 宣言を廃止し `use kekekabu::*` に変更
- **tests/*.rs**: 全 3 テストファイルを SqliteClient ベースに移行
- **Cargo.toml**: `refinery` + `refinery-core` 依存追加
- **migrations/V1__initial_schema.sql**: 新規（schema.rs からの移行）
