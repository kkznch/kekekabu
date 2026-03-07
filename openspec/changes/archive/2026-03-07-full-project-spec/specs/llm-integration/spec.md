## Purpose

LLM バックエンド抽象化。4 バックエンド（api-anthropic, api-gemini, cli-claude, cli-gemini）の統一インターフェースを提供する。

## Requirements

### Requirement: LLM バックエンドの抽象化
システムは SHALL `LlmBackend` トレイトに `send_message(prompt, max_tokens)` メソッドを定義し、全バックエンドが実装する。

#### Scenario: 設定によるバックエンド選択
- **WHEN** 設定で `llm.fetch = "cli-gemini"` や `llm.eval = "cli-claude"` が指定されている場合
- **THEN** `create_backend()` ファクトリ関数で対応するバックエンドを生成する

### Requirement: api-anthropic バックエンド
システムは SHALL Anthropic Messages API を LLM バックエンド（`api-anthropic`）としてサポートする。

#### Scenario: API 呼び出し成功
- **WHEN** 有効な API キーで `api-anthropic` バックエンドがメッセージを送信した場合
- **THEN** 適切なヘッダー（`x-api-key`, `anthropic-version`）付きで `POST https://api.anthropic.com/v1/messages` を呼び出し、テキスト応答を返す

#### Scenario: API キー未設定
- **WHEN** `api-anthropic` が選択されているが `anthropic_api_key` が未設定の場合
- **THEN** ANTHROPIC_API_KEY が必要である旨のエラーを返す

### Requirement: api-gemini バックエンド
システムは SHALL Google Gemini generateContent API を LLM バックエンド（`api-gemini`）としてサポートする。

#### Scenario: API 呼び出し成功
- **WHEN** 有効な API キーで `api-gemini` バックエンドがメッセージを送信した場合
- **THEN** `POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent` を呼び出し、テキスト応答を返す

#### Scenario: API キー未設定
- **WHEN** `api-gemini` が選択されているが `gemini_api_key` が未設定の場合
- **THEN** GEMINI_API_KEY が必要である旨のエラーを返す

### Requirement: cli-claude バックエンド
システムは SHALL Claude CLI（`claude -p`）を LLM バックエンド（`cli-claude`）としてサポートする。

#### Scenario: CLI 未インストール
- **WHEN** `cli-claude` が選択されているが `claude` コマンドが PATH に見つからない場合
- **THEN** claude CLI がインストールされていない旨のエラーを返す

#### Scenario: CLI 呼び出し成功
- **WHEN** `cli-claude` バックエンドがメッセージを送信した場合
- **THEN** `claude -p "<prompt>"`（オプションで `--model` フラグ付き）を実行し、stdout を返す

### Requirement: cli-gemini バックエンド
システムは SHALL Gemini CLI（`gemini -p`）を LLM バックエンド（`cli-gemini`）としてサポートする。

#### Scenario: CLI 未インストール
- **WHEN** `cli-gemini` が選択されているが `gemini` コマンドが PATH に見つからない場合
- **THEN** gemini CLI がインストールされていない旨のエラーを返す

#### Scenario: CLI 呼び出し成功
- **WHEN** `cli-gemini` バックエンドがメッセージを送信した場合
- **THEN** `gemini -p "<prompt>"`（オプションで `--model` フラグ付き）を実行し、stdout を返す

### Requirement: モデルのオーバーライド
システムは SHALL `fetch_model` / `eval_model` 設定で各バックエンドのデフォルトモデルを変更できる。

#### Scenario: カスタムモデルの指定
- **WHEN** `eval_model = "claude-opus-4-5-20250514"` が設定されている場合
- **THEN** eval 時にデフォルトモデルの代わりに指定されたモデルを使用する
