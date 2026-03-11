## MODIFIED Requirements

### Requirement: LLM バックエンドの抽象化
システムは SHALL `LlmBackend` トレイトに `send_message(prompt, max_tokens, temperature)` メソッドを定義し、全バックエンドが実装する。`temperature` は `Option<f32>` 型とし、`None` の場合は API デフォルトを使用する。`send_message_with_schema` にも同様に `temperature` パラメータを追加する。

#### Scenario: 設定によるバックエンド選択
- **WHEN** 設定で `llm.fetch = "cli-gemini"` や `llm.eval = "cli-claude"` が指定されている場合
- **THEN** `create_backend()` ファクトリ関数で対応するバックエンドを生成する

#### Scenario: temperature 指定付きメッセージ送信
- **WHEN** `send_message(prompt, max_tokens, Some(0.0))` が呼び出された場合
- **THEN** API バックエンドはリクエストに `temperature: 0.0` を含める

#### Scenario: temperature 未指定のメッセージ送信
- **WHEN** `send_message(prompt, max_tokens, None)` が呼び出された場合
- **THEN** API バックエンドはリクエストに temperature フィールドを含めず、API のデフォルト値を使用する

### Requirement: api-anthropic バックエンド
システムは SHALL Anthropic Messages API を LLM バックエンド（`api-anthropic`）としてサポートする。temperature が指定された場合はリクエストボディに含める。

#### Scenario: temperature 付き API 呼び出し
- **WHEN** `api-anthropic` バックエンドが `temperature: Some(0.0)` でメッセージを送信した場合
- **THEN** リクエスト JSON に `"temperature": 0.0` を含める

#### Scenario: API キー未設定
- **WHEN** `api-anthropic` が選択されているが `anthropic_api_key` が未設定の場合
- **THEN** ANTHROPIC_API_KEY が必要である旨のエラーを返す

### Requirement: api-gemini バックエンド
システムは SHALL Google Gemini generateContent API を LLM バックエンド（`api-gemini`）としてサポートする。temperature が指定された場合は generationConfig に含める。

#### Scenario: temperature 付き API 呼び出し
- **WHEN** `api-gemini` バックエンドが `temperature: Some(0.0)` でメッセージを送信した場合
- **THEN** generationConfig に `"temperature": 0.0` を含める

#### Scenario: API キー未設定
- **WHEN** `api-gemini` が選択されているが `gemini_api_key` が未設定の場合
- **THEN** GEMINI_API_KEY が必要である旨のエラーを返す

### Requirement: cli-claude バックエンド
システムは SHALL Claude CLI（`claude -p`）を LLM バックエンド（`cli-claude`）としてサポートする。temperature が指定された場合は警告ログを出して無視する。

#### Scenario: temperature 指定時の警告
- **WHEN** `cli-claude` バックエンドが `temperature: Some(0.0)` でメッセージを送信した場合
- **THEN** CLI は temperature 制御をサポートしない旨の warn ログを出力し、temperature なしで実行する

#### Scenario: CLI 未インストール
- **WHEN** `cli-claude` が選択されているが `claude` コマンドが PATH に見つからない場合
- **THEN** claude CLI がインストールされていない旨のエラーを返す

### Requirement: cli-gemini バックエンド
システムは SHALL Gemini CLI（`gemini -p`）を LLM バックエンド（`cli-gemini`）としてサポートする。temperature が指定された場合は警告ログを出して無視する。

#### Scenario: temperature 指定時の警告
- **WHEN** `cli-gemini` バックエンドが `temperature: Some(0.0)` でメッセージを送信した場合
- **THEN** CLI は temperature 制御をサポートしない旨の warn ログを出力し、temperature なしで実行する

#### Scenario: CLI 未インストール
- **WHEN** `cli-gemini` が選択されているが `gemini` コマンドが PATH に見つからない場合
- **THEN** gemini CLI がインストールされていない旨のエラーを返す
