## MODIFIED Requirements

### Requirement: 買い取引の記録
システムは SHALL 買い取引を記録し、加重平均コストでポートフォリオポジションを更新する。この機能は内部関数として提供し、CLI コマンドとしては公開しない。

#### Scenario: 初回の買いで新規ポジション作成
- **WHEN** 内部関数 `portfolio::buy()` が ticker=7203, quantity=100, price=2000 で呼ばれた場合
- **THEN** quantity=100, avg_cost=2000 の新規ポジションを作成する

#### Scenario: 買い増しで平均コストを更新
- **WHEN** 100株を2000円で購入済みの銘柄に対して、さらに100株を2200円で `portfolio::buy()` が呼ばれた場合
- **THEN** ポジションを quantity=200, avg_cost=2100（加重平均）に更新する

### Requirement: 売り取引の記録
システムは SHALL 売り取引を記録し、P&L を計算してポジションを更新する。ポジションがゼロになった場合は watchlist からの自動除外を行う。この機能は内部関数として提供し、CLI コマンドとしては公開しない。

#### Scenario: 一部売却
- **WHEN** avg_cost=2000 の100株ポジションから50株を2200円で `portfolio::sell()` が呼ばれた場合
- **THEN** ポジションを quantity=50 に更新し、pnl=(2200-2000)*50=10000 の取引を記録する。watchlist からは削除しない。

#### Scenario: 全株売却でポジションクローズと watchlist 自動除外
- **WHEN** ポジションの全株数に対して `portfolio::sell()` が呼ばれた場合
- **THEN** ポジションの is_active=0 に設定し、watchlist から当該銘柄を削除し、watchlist_events に auto-removed-on-sell を記録する

### Requirement: 保有ポジション一覧
システムは SHALL アクティブなポートフォリオポジションの一覧を表示する。

#### Scenario: ポジション出力
- **WHEN** `kabu show positions` を実行した場合
- **THEN** アクティブなポジションを JSON で出力する（ticker, name, quantity, avg_cost, unrealized_pnl）

### Requirement: ポートフォリオサマリー
システムは SHALL ポートフォリオの集約サマリーを提供する。

#### Scenario: サマリーの算出
- **WHEN** `kabu show summary` を実行した場合
- **THEN** position_count, total_invested, total_current_value, total_unrealized_pnl, total_unrealized_pnl_pct を出力する

### Requirement: 取引履歴
システムは SHALL 過去の取引一覧を提供する。

#### Scenario: 件数制限付き取引履歴
- **WHEN** `kabu show trades --limit 20` を実行した場合
- **THEN** 直近20件の取引を出力する（ticker, side, date, quantity, price, pnl）

## REMOVED Requirements

### Requirement: portfolio buy CLI コマンド
**Reason**: 全自動パイプラインでは手動売買を行わない。買い取引は execute の Tachibana API 統合時に内部関数として呼ばれる。
**Migration**: `kabu portfolio buy` は使用不可。内部関数 `portfolio::buy()` は残存。

### Requirement: portfolio sell CLI コマンド
**Reason**: 全自動パイプラインでは手動売買を行わない。売り取引は execute の Tachibana API 統合時に内部関数として呼ばれる。
**Migration**: `kabu portfolio sell` は使用不可。内部関数 `portfolio::sell()` は残存。
