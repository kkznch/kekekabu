## Purpose

LLM 判断に依存しないルールベースの強制損切り機構。投資 Spec の `[execution].stop_loss` 閾値を超える損失ポジションを自動で成行売りし、`[execution].max_position_size` を超える買い注文を拒否する。execute コマンドの安全機構として circuit breaker と並列して機能する。

## Requirements

### Requirement: ハードストップロスによる強制売り
システムは SHALL execute 実行時に全保有ポジションの損失率を確認し、投資 Spec の `[execution].stop_loss` 閾値以下のポジションを LLM の eval 判断に関わらず成行注文で強制売りする。

#### Scenario: stop_loss 閾値を超える損失ポジション
- **WHEN** ポジションの `unrealized_pnl_pct` が spec の `stop_loss`（例: -7.0%）以下である場合
- **THEN** 当該ポジションの全数量を成行注文（`force_market: true`）で売り、`HardStopLossAction` として結果に記録する

#### Scenario: stop_loss が spec に未定義
- **WHEN** 投資 Spec に `[execution].stop_loss` が定義されていない場合
- **THEN** ハードストップロス判定をスキップし、従来通り eval ベースの判断のみで execute する

#### Scenario: eval の Sell と stop-loss が同一銘柄で競合
- **WHEN** eval が Sell シグナルを出しており、同時に stop-loss もトリガーされた場合
- **THEN** eval の Sell シグナルを優先し、stop-loss の強制売り注入をスキップする

#### Scenario: stop-loss 対象銘柄への Buy シグナルのブロック
- **WHEN** stop-loss がトリガーされた銘柄に対して eval が Buy シグナルを出した場合
- **THEN** Buy シグナルを `blocked_by_stop_loss` として無効化する

### Requirement: 最大ポジションサイズによる買い注文制限
システムは SHALL 買い注文の金額が投資 Spec の `[execution].max_position_size` × `[budget].initial_cash` を超える場合、注文を reject する。

#### Scenario: max_position_size を超える買い注文
- **WHEN** 買い注文金額（`last_close × quantity`）が `initial_cash × max_position_size` を超える場合
- **THEN** 注文を reject し、`order_results` に `rejected: exceeds max position size` として記録する

#### Scenario: max_position_size または initial_cash が未定義
- **WHEN** spec に `max_position_size` または `initial_cash` が定義されていない場合
- **THEN** 最大ポジションサイズチェックをスキップする

### Requirement: InvestmentSpec の execution パラメータアクセサ
システムは SHALL `InvestmentSpec` に `execution_stop_loss()` と `execution_max_position_size()` メソッドを提供し、spec TOML の `[execution]` セクションから数値を読み取る。

#### Scenario: execution パラメータの読み取り
- **WHEN** spec TOML に `[execution].stop_loss = -0.07` が定義されている場合
- **THEN** `execution_stop_loss()` が `Some(-0.07)` を返す

#### Scenario: execution セクションが存在しない場合
- **WHEN** spec TOML に `[execution]` セクションが存在しない場合
- **THEN** `execution_stop_loss()` と `execution_max_position_size()` が `None` を返す
