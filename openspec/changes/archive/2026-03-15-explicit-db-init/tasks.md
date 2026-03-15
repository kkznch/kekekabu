## 1. DB 層の変更

- [x] 1.1 `SqliteClient::open()` に DB ファイル存在チェックを追加（なければエラー + 案内メッセージ）
- [x] 1.2 `SqliteClient::open_or_create()` を新設（DB 作成 + マイグレーション）
- [x] 1.3 共通ロジックを `open_and_migrate()` に抽出

## 2. コマンドの接続

- [x] 2.1 `cmd/db.rs` の `migrate()` で `open_or_create()` を使用
- [x] 2.2 `cmd/db.rs` の `reset()` の案内メッセージを更新

## 3. 検証

- [x] 3.1 `just ci` で全テスト通過を確認
- [x] 3.2 DB なしで `show tables` がエラーになることを確認
- [x] 3.3 `db migrate` で DB が作成されることを確認
- [x] 3.4 作成後に `show tables` が正常動作することを確認
