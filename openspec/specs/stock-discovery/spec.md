## Purpose

LLM を活用した有望銘柄の自動発掘と watchlist の差分管理。投資 Spec に基づいて Gemini CLI で候補銘柄を発掘し、watchlist を自動更新する。

## Requirements

### Requirement: Gemini CLI による自動銘柄発掘
システムは SHALL `kabu discover` 実行時に、投資 Spec の条件と現在のウォッチリストに基づいて Gemini CLI で有望な日本株銘柄を発掘し、watchlist テーブルを自動更新する。プロンプトには立花証券と同期済みの実残高（`account_balance` の最新スナップショット）を Budget Context として必ず含め、資金状況を考慮した銘柄選定を行う。プロンプトには現在のウォッチリスト銘柄を含め、LLM に keep/add/remove のアクション別判断を求める。

#### Scenario: 銘柄発掘の成功
- **WHEN** 有効な投資 Spec が設定された状態で `kabu discover` を実行した場合
- **THEN** Gemini CLI に投資 Spec と現在のウォッチリストを含むプロンプトを送信し、keep/add/remove のアクション別銘柄リストを JSON で受け取り、watchlist テーブルを更新する

#### Scenario: ウォッチリストが空の状態での発掘
- **WHEN** watchlist が空の状態で `kabu discover` を実行した場合
- **THEN** 「現在の追跡銘柄なし」としてプロンプトを構築し、add のみの結果を返す

#### Scenario: 同期済み残高を使った銘柄発掘
- **WHEN** `account_balance` テーブルに残高スナップショットがある状態で `kabu discover` を実行した場合
- **THEN** プロンプトに「Cash Available」（実残高）と同期日時を含む Budget Context セクションを含め、購入可能性を考慮した銘柄を選定させる

#### Scenario: 未同期での discover 実行
- **WHEN** `account_balance` テーブルが空の状態で `kabu discover` を実行した場合
- **THEN** 「Run `kabu sync` first」のエラーメッセージで異常終了する

#### Scenario: LLM 判断に基づく差分管理
- **WHEN** discover が keep/add/remove のアクション別レスポンスを返した場合
- **THEN** add の銘柄を watchlist に追加し、remove の銘柄を watchlist から削除する。ただし portfolio_positions に保有中の銘柄は remove でも削除しない

#### Scenario: 不正な ticker の無視
- **WHEN** Gemini CLI が不正な形式の ticker（数字4〜5桁でない）を返した場合
- **THEN** その ticker をスキップし、警告をログ出力する

#### Scenario: add/keep 間の重複排除
- **WHEN** 同一 ticker が add リストと keep リストの両方に含まれている場合
- **THEN** 重複を排除し、1銘柄として処理する

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
システムは SHALL Gemini CLI の応答を keep/add/remove のアクション別 ticker リストとしてパースする。

#### Scenario: 正常な JSON パース
- **WHEN** Gemini CLI が keep/add/remove を含む有効な JSON を返した場合
- **THEN** 各アクションの ticker を抽出し、watchlist の更新に使用する

#### Scenario: パース失敗
- **WHEN** Gemini CLI の応答が有効な JSON でない場合
- **THEN** エラーを返し、watchlist を変更しない
