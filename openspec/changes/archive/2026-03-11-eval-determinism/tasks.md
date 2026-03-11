## 1. LlmBackend トレイトに temperature パラメータを追加

- [x] 1.1 `send_message` に `temperature: Option<f32>` パラメータを追加し、全バックエンドのシグネチャを更新
- [x] 1.2 `send_message_with_schema` にも `temperature: Option<f32>` パラメータを追加
- [x] 1.3 api-anthropic の `MessageRequest` に `temperature` フィールドを追加し、`Some` の場合のみ JSON に含める
- [x] 1.4 api-gemini の `GenerationConfig` に `temperature` フィールドを追加し、`Some` の場合のみ JSON に含める
- [x] 1.5 cli-claude / cli-gemini で `temperature: Some(...)` 指定時に warn ログを出して無視する
- [x] 1.6 既存の全呼び出し箇所（eval, fetch, discover, workflow）を `temperature: None` で更新

## 2. eval で temperature=0 を指定

- [x] 2.1 `cmd/eval.rs` の LLM 呼び出しで `temperature: Some(0.0)` を指定
- [x] 2.2 `cmd/workflow.rs` の eval ステップでも `temperature: Some(0.0)` を指定

## 3. LLM ログの DB 保存

- [x] 3.1 `db/schema.rs` に `llm_logs` テーブル定義を追加し、`ALL_SCHEMAS` に登録
- [x] 3.2 `db/mod.rs` に `save_llm_log(conn, command, ticker, backend, model, temperature, prompt, response)` を実装
- [x] 3.3 `cmd/eval.rs` で LLM 呼び出し後にログを保存
- [x] 3.4 `cmd/fetch.rs` で LLM 呼び出し後にログを保存
- [x] 3.5 `cmd/discover.rs` で LLM 呼び出し後にログを保存
- [x] 3.6 `cmd/workflow.rs` の各ステップ（fetch, eval）でもログを保存

## 4. LLM ログの閲覧コマンド

- [x] 4.1 `db/mod.rs` に `list_llm_logs(conn, limit, ticker)` を実装
- [x] 4.2 `cmd/show.rs` に `llm_logs` サブコマンドを追加
- [x] 4.3 `main.rs` の `ShowCommand` に `LlmLogs` バリアントを追加してルーティング

## 5. テスト

- [x] 5.1 temperature パラメータ付きバックエンド呼び出しの単体テスト
- [x] 5.2 `llm_logs` テーブルの保存・取得の統合テスト
- [x] 5.3 既存テストが全て通ることを確認
