## Purpose

LLM による投資判断生成。TA 指標・収集情報・投資 Spec を統合したプロンプトで、Hunting（新規候補: Buy/Avoid）と Farming（保有中: Hold/Sell）の評価を行う。

## Requirements

### Requirement: LLM による投資判断の生成
システムは SHALL TA 指標、fetch 結果、投資 Spec を統合した包括的なプロンプトを構築し、eval 用 LLM バックエンドに `temperature=0` を指定して送信して投資判断を生成する。対象は watchlist の新規候補（Hunting）と portfolio_positions の保有中銘柄（Farming）の両方とする。プロンプトには立花証券と同期済みの実残高を Budget Context として必ず含め、資金状況を考慮した判断を行う。

#### Scenario: 新規候補（Hunting）の評価
- **WHEN** watchlist にあるが portfolio_positions に保有していない銘柄に対して `kabu eval` を実行した場合
- **THEN** status="NewTarget" として各銘柄を `temperature=0` で評価し、Buy/Avoid の判断を生成する

#### Scenario: 保有中銘柄（Farming）の評価
- **WHEN** portfolio_positions に保有中の銘柄に対して `kabu eval` を実行した場合
- **THEN** status="ExistingHolding" として各銘柄を `temperature=0` で評価し、Hold/Sell の判断を生成する

#### Scenario: 特定銘柄の評価
- **WHEN** `kabu eval 7203` を実行した場合
- **THEN** 指定された銘柄のみを `temperature=0` で評価する（watchlist と portfolio_positions の両方から検索）

#### Scenario: 同期済み残高を使った評価
- **WHEN** `account_balance` テーブルに残高スナップショットがある状態で `kabu eval` を実行した場合
- **THEN** プロンプトに「Cash Available」（実残高）と同期日時を含む Budget Context セクションを含めて `temperature=0` で送信する

#### Scenario: 未同期での eval 実行
- **WHEN** `account_balance` テーブルが空の状態で `kabu eval` を実行した場合
- **THEN** 「Run `kabu sync` first」のエラーメッセージで異常終了する

### Requirement: 評価応答のフォーマット
システムは SHALL LLM 応答を `status`（NewTarget/ExistingHolding）、`decision`（Buy/Hold/Sell/Avoid）、`score`（0-100）、`analysis`（catalyst_check, risk_assessment, spec_compliance）、`execution_instruction`（action, reason_for_exit）を含む JSON としてパースする。

#### Scenario: 有効な eval 応答
- **WHEN** LLM が適切なフォーマットの JSON 応答を返した場合
- **THEN** status, decision, score, analysis, execution_instruction フィールドを抽出する

#### Scenario: Markdown でラップされた eval 応答
- **WHEN** LLM が markdown コードブロックでラップされた JSON を返した場合
- **THEN** パース前にコードブロックから JSON を抽出する

### Requirement: eval プロンプトに投資 Spec を含める
システムは SHALL eval プロンプトに投資 Spec（ユニバースフィルタ、スコアリング要因、執行パラメータ）を含める。

#### Scenario: Spec のプロンプト埋め込み
- **WHEN** Spec ファイルが設定された状態で eval コマンドを実行した場合
- **THEN** Spec TOML を読み込み、プロンプトセクションに変換して LLM プロンプトに含める

### Requirement: eval プロンプトに Hunting/Farming のコンテキストを含める
システムは SHALL eval プロンプトに、対象銘柄が新規候補（Hunting）か保有中（Farming）かの区分と、保有中銘柄の場合はポジション情報（取得単価、数量、損益率）を含める。

#### Scenario: Farming 銘柄のプロンプト
- **WHEN** portfolio_positions に保有中の銘柄を評価する場合
- **THEN** プロンプトに保有数量、平均取得単価、現在の損益率を含め、シナリオ維持/崩壊の判断を求める

#### Scenario: Hunting 銘柄のプロンプト
- **WHEN** watchlist にあるが未保有の銘柄を評価する場合
- **THEN** プロンプトに「新規投資候補」であることを明示し、Buy/Avoid の判断を求める

### Requirement: 評価結果を Spec ハッシュ付きで永続化
システムは SHALL 評価結果を、使用した Spec の SHA256 ハッシュ付きでデータベースに保存する。

#### Scenario: spec_hash 付き評価保存
- **WHEN** 評価が完了した場合
- **THEN** ticker, name, decision, score, rationale, spec_hash, evaluated_at を `evaluations` テーブルに保存する
