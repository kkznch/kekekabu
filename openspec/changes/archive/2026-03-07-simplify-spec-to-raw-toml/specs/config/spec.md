## MODIFIED Requirements

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
