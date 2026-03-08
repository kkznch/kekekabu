## MODIFIED Requirements

### Requirement: LLM バックエンドの抽象化
システムは SHALL `LlmBackend` トレイトに `send_message(prompt, max_tokens)` メソッドを定義し、全バックエンドが実装する。さらに `send_message_with_schema(prompt, max_tokens, tool_name, tool_description, schema)` メソッドをデフォルト実装付きで提供する。

#### Scenario: 設定によるバックエンド選択
- **WHEN** 設定で `llm.fetch = "cli-gemini"` や `llm.eval = "cli-claude"` が指定されている場合
- **THEN** `create_backend()` ファクトリ関数で対応するバックエンドを生成する

#### Scenario: スキーマ付きメッセージ送信のデフォルト動作
- **WHEN** `send_message_with_schema()` がデフォルト実装のバックエンドで呼ばれた場合
- **THEN** スキーマを無視して `send_message()` にフォールバックし、プレーンテキストを返す

### Requirement: api-anthropic バックエンド
システムは SHALL Anthropic Messages API を LLM バックエンド（`api-anthropic`）としてサポートする。`send_message_with_schema()` が呼ばれた場合は tool_use による構造化出力を返す。

#### Scenario: API 呼び出し成功
- **WHEN** 有効な API キーで `api-anthropic` バックエンドがメッセージを送信した場合
- **THEN** 適切なヘッダー（`x-api-key`, `anthropic-version`）付きで `POST https://api.anthropic.com/v1/messages` を呼び出し、テキスト応答を返す

#### Scenario: API キー未設定
- **WHEN** `api-anthropic` が選択されているが `anthropic_api_key` が未設定の場合
- **THEN** ANTHROPIC_API_KEY が必要である旨のエラーを返す

#### Scenario: tool_use による構造化出力
- **WHEN** `send_message_with_schema()` が tool_name, tool_description, JSON Schema 付きで呼ばれた場合
- **THEN** リクエストに `tools` と `tool_choice` フィールドを含め、レスポンスの `tool_use` コンテンツブロックから `input` フィールドを JSON 文字列として返す

#### Scenario: tool_use レスポンスに tool_use ブロックがない
- **WHEN** API レスポンスに `tool_use` タイプのコンテンツブロックが含まれない場合
- **THEN** テキストコンテンツにフォールバックして返す
