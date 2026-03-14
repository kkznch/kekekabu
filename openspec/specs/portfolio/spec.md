## Purpose

ポートフォリオ管理。売買記録、加重平均コスト計算、P&L 算出、ポジション追跡を行い、投資の全履歴を保持する。

## Requirements

### Requirement: 買い取引の記録
システムは SHALL 買い取引を記録し、加重平均コストでポートフォリオポジションを更新する。

#### Scenario: 初回の買いで新規ポジション作成
- **WHEN** `kabu portfolio buy 7203 --quantity 100 --price 2000` を実行した場合
- **THEN** quantity=100, avg_cost=2000 の新規ポジションを作成する

#### Scenario: 買い増しで平均コストを更新
- **WHEN** 100株を2000円で購入後、さらに100株を2200円で購入した場合
- **THEN** ポジションを quantity=200, avg_cost=2100（加重平均）に更新する

### Requirement: 売り取引の記録
システムは SHALL 売り取引を記録し、P&L を計算してポジションを更新する。

#### Scenario: 一部売却
- **WHEN** avg_cost=2000 の100株ポジションから50株を2200円で売却した場合
- **THEN** ポジションを quantity=50 に更新し、pnl=(2200-2000)*50=10000 の取引を記録する

#### Scenario: 全株売却でポジションクローズ
- **WHEN** ポジションの全株数を売却した場合
- **THEN** ポジションの is_active=0 に設定する（クローズ）

#### Scenario: 売却済み銘柄の再購入
- **WHEN** 過去に全株売却（is_active=0）した銘柄を再度購入した場合
- **THEN** 既存のクローズ済みポジションを再活性化（is_active=1）し、新しい quantity と avg_cost で更新する

### Requirement: 保有ポジション一覧
システムは SHALL アクティブなポートフォリオポジションの一覧を表示する。

#### Scenario: ポジション出力
- **WHEN** `kabu portfolio positions` を実行した場合
- **THEN** アクティブなポジションを JSON で出力する（ticker, name, quantity, avg_cost, unrealized_pnl）

### Requirement: ポートフォリオサマリー
システムは SHALL ポートフォリオの集約サマリーを提供する。

#### Scenario: サマリーの算出
- **WHEN** `kabu portfolio summary` を実行した場合
- **THEN** position_count, total_invested, total_current_value, total_unrealized_pnl, total_unrealized_pnl_pct を出力する

### Requirement: 取引履歴
システムは SHALL 過去の取引一覧を提供する。

#### Scenario: 件数制限付き取引履歴
- **WHEN** `kabu portfolio trades --limit 20` を実行した場合
- **THEN** 直近20件の取引を出力する（ticker, side, date, quantity, price, pnl）
