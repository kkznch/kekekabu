## Why

従来 `SqliteClient::open()` は DB ファイルが存在しない場合に暗黙的に新規作成していた。これにより typo で別パスの config を参照した場合などに空 DB が作られ、「データがない」と誤認するリスクがあった。DB の作成は明示的に `kabu db migrate` でのみ行うようにし、安全性を向上させる。

## What Changes

- `SqliteClient::open()` は DB ファイルが存在しない場合にエラーを返す（`kabu db migrate` を案内）
- `SqliteClient::open_or_create()` を新設し、`kabu db migrate` のみがこれを呼ぶ
- 内部共通ロジックを `open_and_migrate()` に抽出
- `kabu db reset` の案内メッセージを更新

## Capabilities

### New Capabilities

（なし）

### Modified Capabilities
- `database`: `open()` が DB 不在時にエラーを返すように変更。`open_or_create()` を新設

## Impact

- `src/db/mod.rs` — `open()` / `open_or_create()` / `open_and_migrate()` の3メソッド構成に変更
- `src/cmd/db.rs` — `migrate()` が `open_or_create()` を使用、`reset()` のメッセージ更新
