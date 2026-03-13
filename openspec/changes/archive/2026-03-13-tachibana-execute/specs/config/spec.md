## MODIFIED Requirements

### Requirement: ~/.config/kabu/config.toml による設定管理
システムは SHALL `~/.config/kabu/config.toml` から [api], [llm], [spec], [tachibana] セクションの設定を読み込む。

#### Scenario: 設定ファイルの正常読み込み
- **WHEN** 有効な TOML 形式の config.toml が存在する場合
- **THEN** API キー、LLM バックエンド設定、Spec パス、出力形式、立花証券接続情報を読み込む

#### Scenario: 設定ファイルが存在しない場合
- **WHEN** config.toml が存在しない場合
- **THEN** デフォルト値を使用する（fetch=cli-gemini, eval=cli-claude, spec=specs/template.toml, format=json）

## ADDED Requirements

### Requirement: 立花証券 API 接続設定
システムは SHALL [tachibana] セクションで立花証券 e支店 API の接続情報を管理する。

#### Scenario: config.toml からの読み込み
- **WHEN** config.toml に [tachibana] セクションが定義されている場合
- **THEN** user_id, password, second_password, event_timeout_secs を読み込む

#### Scenario: 環境変数による上書き
- **WHEN** TACHIBANA_USER_ID, TACHIBANA_PASSWORD, TACHIBANA_SECOND_PASSWORD 環境変数が設定されている場合
- **THEN** config.toml の値を環境変数で上書きする

#### Scenario: tachibana 設定が未定義の場合
- **WHEN** [tachibana] セクションが存在しない状態で execute を dry-run なしで実行した場合
- **THEN** 立花証券 API の接続情報が必要である旨のエラーを返す

#### Scenario: dry-run では tachibana 設定不要
- **WHEN** [tachibana] セクションが存在しない状態で execute を dry-run で実行した場合
- **THEN** 従来どおりシグナル生成のみを行い、エラーは発生しない
