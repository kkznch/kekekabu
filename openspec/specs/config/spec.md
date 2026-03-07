## Purpose

TOML 設定ファイルと環境変数によるアプリケーション設定管理、および config サブコマンドによる初期設定・バリデーション・テンプレート生成。

## Requirements

### Requirement: ~/.config/kabu/config.toml による設定管理
システムは SHALL `~/.config/kabu/config.toml` から [api], [llm], [spec] セクションの設定を読み込む。

#### Scenario: 設定ファイルの正常読み込み
- **WHEN** 有効な TOML 形式の config.toml が存在する場合
- **THEN** API キー、LLM バックエンド設定、Spec パス、出力形式を読み込む

#### Scenario: 設定ファイルが存在しない場合
- **WHEN** config.toml が存在しない場合
- **THEN** デフォルト値を使用する（fetch=cli-gemini, eval=cli-claude, spec=specs/template.toml, format=json）

### Requirement: 環境変数による設定の上書き
システムは SHALL JQUANTS_API_KEY, ANTHROPIC_API_KEY, GEMINI_API_KEY の環境変数で設定値を上書きできる。

#### Scenario: 環境変数の優先
- **WHEN** config.toml と環境変数の両方で jquants_api_key が定義されている場合
- **THEN** 環境変数の値が優先される

#### Scenario: 空の環境変数は無視
- **WHEN** 環境変数が空文字列に設定されている場合
- **THEN** 環境変数を無視し config.toml の値を使用する

### Requirement: config init コマンドで設定と Spec テンプレートを生成
システムは SHALL `kabu config init` 実行時に config.toml と specs/template.toml を生成する。

#### Scenario: 初回の init
- **WHEN** 既存の設定がない状態で `kabu config init` を実行した場合
- **THEN** API キーのプレースホルダー付き config.toml と specs/template.toml を作成する

#### Scenario: 既存設定がある場合
- **WHEN** 既存の config.toml がある状態で `kabu config init` を実行した場合
- **THEN** `--force` での上書きを促すエラーを返す

#### Scenario: 強制上書き
- **WHEN** `kabu config init --force` を実行した場合
- **THEN** config.toml を上書きし specs/template.toml を再生成する

### Requirement: Spec テンプレートは常に上書き
システムは SHALL init 時に `specs/template.toml` を常に上書きする。ユーザーのカスタム戦略は別ファイルで管理する想定。

#### Scenario: テンプレートの再生成
- **WHEN** `kabu config init`（または `--force`）を実行した場合
- **THEN** template.toml の存在有無にかかわらず最新版を書き出す

### Requirement: config validate コマンドで設定をバリデーション
システムは SHALL `kabu config validate` 実行時に config.toml と投資 Spec TOML の両方をバリデーションする。Spec のバリデーションは TOML 構文の正当性と `name` キーの存在確認のみとする。

#### Scenario: 正常なバリデーション
- **WHEN** config.toml と Spec TOML が両方とも有効な場合
- **THEN** 「Config: OK」「Spec (<name>): OK」「All validations passed.」を stderr に出力する

#### Scenario: Spec が無効な TOML の場合
- **WHEN** Spec ファイルが TOML として構文エラーを含む場合
- **THEN** 「Spec: FAILED」と TOML パースエラーの詳細を返す

#### Scenario: Spec に name キーがない場合
- **WHEN** Spec ファイルが有効な TOML だが `name` キーが存在しない場合
- **THEN** 「Spec: FAILED」と「Spec file must have a 'name' field」エラーを返す

#### Scenario: Spec の構造が自由形式の場合
- **WHEN** Spec ファイルに任意のセクション（universe.liquidity, quantitative.value 等）が含まれる場合
- **THEN** 構造に関わらず TOML 構文が有効で `name` があればバリデーション通過する
