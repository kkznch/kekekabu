## Why

同じ入力データに対して eval の LLM 出力が揺れるため、評価の再現性が低い。temperature 制御がなく、プロンプト/レスポンスのログも残らないためデバッグや判断根拠の事後検証ができない。

## What Changes

- LlmBackend トレイトに temperature パラメータを追加し、eval 呼び出し時に temperature=0 を指定可能にする
- API バックエンド（api-anthropic, api-gemini）のリクエストに temperature フィールドを含める
- CLI バックエンド（cli-claude, cli-gemini）は温度制御不可のためログで警告を出す
- LLM のプロンプト/レスポンスを DB に保存し、判断根拠の事後検証・デバッグを可能にする

## Capabilities

### New Capabilities

- `llm-logging`: LLM プロンプト/レスポンスの永続化ログ（DB テーブル + 閲覧コマンド）

### Modified Capabilities

- `llm-integration`: send_message / send_message_with_schema に temperature パラメータを追加
- `investment-evaluation`: eval 呼び出し時に temperature=0 を指定

## Impact

- `src/llm/mod.rs` — LlmBackend トレイトのシグネチャ変更（**BREAKING**: 全バックエンド実装の更新が必要）
- `src/llm/api_anthropic.rs`, `api_gemini.rs` — リクエストに temperature 追加
- `src/llm/cli_claude.rs`, `cli_gemini.rs` — temperature 非対応の警告ログ
- `src/db/schema.rs`, `src/db/mod.rs` — `llm_logs` テーブル追加
- `src/cmd/eval.rs`, `src/cmd/workflow.rs` — temperature=0 指定
- `src/cmd/show.rs` — `kabu show llm-logs` 閲覧コマンド追加
