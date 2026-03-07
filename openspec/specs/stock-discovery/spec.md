## Purpose

LLM を活用した有望銘柄の自動発掘と watchlist の差分管理。投資 Spec に基づいて Gemini CLI で候補銘柄を発掘し、watchlist を自動更新する。

## Requirements

### Requirement: Gemini CLI による自動銘柄発掘
システムは SHALL `kabu discover` 実行時に、投資 Spec の条件に基づいて Gemini CLI で有望な日本株銘柄を発掘し、watchlist テーブルを自動更新する。

#### Scenario: 銘柄発掘の成功
- **WHEN** 有効な投資 Spec が設定された状態で `kabu discover` を実行した場合
- **THEN** Gemini CLI に投資 Spec を含むプロンプトを送信し、有望銘柄のリストを JSON で受け取り、watchlist テーブルに追加する

#### Scenario: 既存銘柄との差分管理
- **WHEN** discover が新たな銘柄リストを返した場合
- **THEN** 新規銘柄は watchlist に追加し、リストから外れた銘柄は watchlist から削除する。ただし portfolio_positions に保有中の銘柄は削除しない

#### Scenario: 不正な ticker の無視
- **WHEN** Gemini CLI が不正な形式の ticker（数字4桁でない）を返した場合
- **THEN** その ticker をスキップし、警告をログ出力する

### Requirement: discover --list による追跡銘柄の確認
システムは SHALL `kabu discover --list` で現在の watchlist（discover が追跡中の銘柄）を一覧表示する。

#### Scenario: 一覧表示
- **WHEN** `kabu discover --list` を実行した場合
- **THEN** watchlist テーブルの全銘柄を JSON で stdout に出力する

### Requirement: discover プロンプトに投資 Spec を含める
システムは SHALL discover プロンプトに投資 Spec のユニバースフィルタ（最低時価総額、最低出来高）とスコアリング要因を含め、Spec に合致した銘柄を発掘する。

#### Scenario: Spec ベースの発掘
- **WHEN** 投資 Spec に min_market_cap=10,000,000,000 が設定されている場合
- **THEN** discover プロンプトにこの条件を含め、時価総額が条件を満たす銘柄のみを候補とする

### Requirement: discover の出力を JSON でパース
システムは SHALL Gemini CLI の応答を ticker コードのリストとしてパースする。

#### Scenario: 正常な JSON パース
- **WHEN** Gemini CLI が有効な JSON 配列（ticker と理由を含む）を返した場合
- **THEN** 各 ticker を抽出し、watchlist の更新に使用する

#### Scenario: パース失敗
- **WHEN** Gemini CLI の応答が有効な JSON でない場合
- **THEN** エラーを返し、watchlist を変更しない
