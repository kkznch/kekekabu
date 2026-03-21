## 1. Environment enum の定義

- [ ] 1.1 `src/config.rs` に `Environment` enum（Production / Demo）を定義
- [ ] 1.2 `TachibanaConfig` に `environment` フィールドを追加（デフォルト: Production）
- [ ] 1.3 環境変数 `TACHIBANA_ENVIRONMENT` で上書き可能にする

## 2. CLI フラグ

- [ ] 2.1 `src/main.rs` に `--demo` グローバルフラグを追加
- [ ] 2.2 `--demo` フラグが指定された場合、config の environment を Demo に上書きする

## 3. API エンドポイントの切り替え

- [ ] 3.1 `src/tachibana/mod.rs` の AUTH_URL const を削除し、`TachibanaConfig` から auth_url() を導出するメソッドを追加
- [ ] 3.2 `TachibanaClient::login()` で `config.auth_url()` を使用するように変更

## 4. DB パスの分離

- [ ] 4.1 `src/db/mod.rs` の `db_path()` に `Environment` 引数を追加
- [ ] 4.2 Demo 時は `kekekabu-demo.db` を返すように変更
- [ ] 4.3 `main.rs` と `cmd/db.rs` の `db_path()` 呼び出し箇所を更新

## 5. 検証

- [ ] 5.1 `just ci` で全テスト通過確認
- [ ] 5.2 `kabu --demo db migrate` でデモ用 DB が作成されることを確認
