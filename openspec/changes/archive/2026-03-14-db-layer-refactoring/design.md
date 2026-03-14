## Context

DB アクセス層は `db::save_stock(conn, ...)` のような自由関数として実装されていた。各コマンドハンドラは `tokio_rusqlite::Connection` を直接受け取り、テスト時にモック差し替えができない密結合な設計だった。スキーマ定義は `schema.rs` にハードコードされ、マイグレーション管理の仕組みがなかった。`main.rs` が `mod` 宣言で全モジュールを再宣言しており、bin/lib の二重コンパイルが発生していた。

## Goals / Non-Goals

**Goals:**
- DB 操作を `DbClient` trait に集約し、DI パターンでテスタビリティを向上させる
- refinery によるスキーママイグレーション管理を導入する
- `main.rs` をライブラリクレート参照方式にし、モジュールの二重コンパイルを解消する
- portfolio の async 関数を DB 層に統合し、トランザクション管理を一元化する

**Non-Goals:**
- モック実装の作成（将来の課題として残す）
- DB スキーマ自体の変更（テーブル構造は変えない）
- 新機能の追加

## Decisions

### Decision 1: Trait + impl 方式（vs 構造体にメソッド直書き）

`DbClient` trait を定義し `SqliteClient` で実装する方式を採用。

- **選択理由**: テスト時にモック実装を差し替え可能。将来的に異なるバックエンド（PostgreSQL 等）への移行も容易
- **代替案**: `SqliteClient` にメソッドを直接実装 → テスト時のモック差し替えが不可能

### Decision 2: トランザクション操作は DbClient のメソッドとして実装

`portfolio_buy`/`portfolio_sell`/`update_order_and_record_fill` のようなトランザクションを必要とする操作を DbClient のメソッドとして実装。sync 関数（`buy_sync`/`sell_sync`）は `pub(crate)` として `conn.call()` クロージャ内で使用。

- **選択理由**: トランザクション境界を DB 層に閉じ込め、呼び出し側がトランザクション管理を意識しない設計
- **代替案**: trait にトランザクションオブジェクトを返すメソッドを追加 → async trait とトランザクションの組み合わせが複雑

### Decision 3: refinery マイグレーション（vs 手動 SQL）

refinery クレートでマイグレーション管理。`SqliteClient::open()` 起動時に自動適用。

- **選択理由**: バージョン管理、適用済みスキップ、ロールバック可能性を備えた業界標準のアプローチ
- **代替案**: `schema.rs` での手動 CREATE TABLE → バージョン管理不可、ALTER TABLE のハンドリングが困難

### Decision 4: main.rs をライブラリクレート参照に変更

`mod db;` 等の宣言を廃止し `use kekekabu::{cmd, config, db, ...}` に変更。

- **選択理由**: bin/lib の二重コンパイル解消、clippy の dead_code 誤検知防止
- **代替案**: `#[cfg(test)]` でテスト専用コードを囲む → integration test から見えない問題

## Risks / Trade-offs

- **[async_trait のオーバーヘッド]** → `#[async_trait]` は vtable ディスパッチとヒープ割り当てが発生するが、DB I/O が律速なため影響は無視できる
- **[trait メソッド数の肥大化]** → 36 メソッドは多いが、論理的なグループ分けでコメント整理済み。将来的にサブトレイトへの分割も可能
- **[sync 関数の pub(crate) 露出]** → `buy_sync`/`sell_sync` が crate 内で直接呼び出し可能だが、実際の使用箇所は SqliteClient 内のみ
