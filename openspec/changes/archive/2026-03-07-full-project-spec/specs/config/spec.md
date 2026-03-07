## Purpose

TOML 設定ファイルと環境変数によるアプリケーション設定管理、および init コマンドによる初期設定・テンプレート生成。

## Requirements

### Requirement: ~/.config/kabu/config.toml による設定管理
システムは SHALL `~/.config/kabu/config.toml` から [api], [llm], [spec], [output] セクションの設定を読み込む。

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

### Requirement: init コマンドで設定と Spec テンプレートを生成
システムは SHALL `kabu init` 実行時に config.toml と specs/template.toml を生成する。

#### Scenario: 初回の init
- **WHEN** 既存の設定がない状態で `kabu init` を実行した場合
- **THEN** API キーのプレースホルダー付き config.toml と specs/template.toml を作成する

#### Scenario: 既存設定がある場合
- **WHEN** 既存の config.toml がある状態で `kabu init` を実行した場合
- **THEN** `--force` での上書きを促すエラーを返す

#### Scenario: 強制上書き
- **WHEN** `kabu init --force` を実行した場合
- **THEN** config.toml を上書きし specs/template.toml を再生成する

### Requirement: Spec テンプレートは常に上書き
システムは SHALL init 時に `specs/template.toml` を常に上書きする。ユーザーのカスタム戦略は別ファイルで管理する想定。

#### Scenario: テンプレートの再生成
- **WHEN** `kabu init`（または `--force`）を実行した場合
- **THEN** template.toml の存在有無にかかわらず最新版を書き出す
