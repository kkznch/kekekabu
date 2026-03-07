## MODIFIED Requirements

### Requirement: Gemini CLI による自動銘柄発掘
システムは SHALL `kabu discover` 実行時に、投資 Spec の条件に基づいて Gemini CLI で有望な日本株銘柄を発掘し、watchlist テーブルを自動更新する。Budget Context が利用可能な場合はプロンプトに含め、資金状況を考慮した銘柄選定を行う。

#### Scenario: 銘柄発掘の成功
- **WHEN** 有効な投資 Spec が設定された状態で `kabu discover` を実行した場合
- **THEN** Gemini CLI に投資 Spec を含むプロンプトを送信し、有望銘柄のリストを JSON で受け取り、watchlist テーブルに追加する

#### Scenario: Budget Context 付きの銘柄発掘
- **WHEN** Spec に `[budget]` セクションが定義された状態で `kabu discover` を実行した場合
- **THEN** プロンプトに Budget Context セクションを含め、残り投資可能額と100株単元での購入可能性を考慮した銘柄を選定させる

#### Scenario: Budget Context なしの銘柄発掘
- **WHEN** Spec に `[budget]` セクションが存在しない状態で `kabu discover` を実行した場合
- **THEN** 従来どおり Budget Context なしでプロンプトを送信する

#### Scenario: 既存銘柄との差分管理
- **WHEN** discover が新たな銘柄リストを返した場合
- **THEN** 新規銘柄は watchlist に追加し、リストから外れた銘柄は watchlist から削除する。ただし portfolio_positions に保有中の銘柄は削除しない

#### Scenario: 不正な ticker の無視
- **WHEN** Gemini CLI が不正な形式の ticker（数字4桁でない）を返した場合
- **THEN** その ticker をスキップし、警告をログ出力する
