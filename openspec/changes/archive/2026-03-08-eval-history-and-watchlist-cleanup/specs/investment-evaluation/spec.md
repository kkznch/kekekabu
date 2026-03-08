## MODIFIED Requirements

### Requirement: LLM による投資判断の生成
システムは SHALL TA 指標、fetch 結果、投資 Spec、および直近の評価履歴（最大3件）を統合した包括的なプロンプトを構築し、eval 用 LLM バックエンドに送信して投資判断を生成する。対象は watchlist の新規候補（Hunting）と portfolio_positions の保有中銘柄（Farming）の両方とする。Budget Context が利用可能な場合はプロンプトに含め、資金状況を考慮した判断を行う。

#### Scenario: 新規候補（Hunting）の評価
- **WHEN** watchlist にあるが portfolio_positions に保有していない銘柄に対して `kabu eval` を実行した場合
- **THEN** status="NewTarget" として各銘柄を評価し、Buy/Avoid の判断を生成する

#### Scenario: 保有中銘柄（Farming）の評価
- **WHEN** portfolio_positions に保有中の銘柄に対して `kabu eval` を実行した場合
- **THEN** status="ExistingHolding" として各銘柄を評価し、Hold/Sell の判断を生成する

#### Scenario: 特定銘柄の評価
- **WHEN** `kabu eval 7203` を実行した場合
- **THEN** 指定された銘柄のみを評価する（watchlist と portfolio_positions の両方から検索）

#### Scenario: Budget Context 付きの評価
- **WHEN** Spec に `[budget]` セクションが定義された状態で `kabu eval` を実行した場合
- **THEN** プロンプトに Budget Context セクションを含め、残り投資可能額を考慮した判断を生成する

#### Scenario: Budget Context なしの評価
- **WHEN** Spec に `[budget]` セクションが存在しない状態で `kabu eval` を実行した場合
- **THEN** 従来どおり Budget Context なしでプロンプトを送信する

#### Scenario: 過去の評価履歴付きプロンプト
- **WHEN** 過去に評価された銘柄に対して eval を実行した場合
- **THEN** プロンプトに直近3件の評価履歴（日付、decision、score、rationale の要約）を含めて LLM に送信する

#### Scenario: 評価履歴なしの初回評価
- **WHEN** 過去に評価されたことがない銘柄に対して eval を実行した場合
- **THEN** 評価履歴セクションを省略またはなしと記載してプロンプトを送信する
