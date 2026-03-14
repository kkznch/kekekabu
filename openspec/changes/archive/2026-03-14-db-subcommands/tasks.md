## 1. DB 層の拡張

- [x] 1.1 `db_path()` を `pub` に変更して外部から DB パスを取得可能にする
- [x] 1.2 `MigrationInfo` 構造体（version, name, applied_on）を追加する
- [x] 1.3 `SqliteClient::migration_status()` メソッドを実装する（refinery_schema_history 参照）

## 2. コマンドハンドラ実装

- [x] 2.1 `src/cmd/db.rs` を作成し `migrate()` ハンドラを実装する
- [x] 2.2 `status()` ハンドラを実装する（DB パス・サイズ・マイグレーション履歴）
- [x] 2.3 `reset()` ハンドラを実装する（ランダム6文字確認コード、--force フラグ対応）
- [x] 2.4 `src/cmd/mod.rs` に `pub mod db` を追加する

## 3. CLI ルーティング

- [x] 3.1 `DbCommand` enum（Migrate, Status, Reset）を `src/main.rs` に追加する
- [x] 3.2 `Command::Db(DbCommand)` バリアントを追加する
- [x] 3.3 config 読み込み前に DB サブコマンドをディスパッチする早期 return を実装する

## 4. 出力対応

- [x] 4.1 `MigrationInfo` の `HumanDisplay` 実装を `src/output.rs` に追加する
- [x] 4.2 `DbStatus` 構造体の `Serialize` + `HumanDisplay` 実装を追加する

## 5. 依存関係

- [x] 5.1 `rand` クレートを `Cargo.toml` に追加する

## 6. 検証

- [x] 6.1 `cargo fmt && just ci` で全テスト通過を確認する
