## Purpose

ポートフォリオ管理。売買記録、加重平均コスト計算、P&L 算出、ポジション追跡を行い、投資の全履歴を保持する。

## Requirements

### Requirement: 買い取引の記録
システムは SHALL 買い取引を記録し、加重平均コストでポートフォリオポジションを更新する。買い操作は `SqliteClient` のメソッド（`portfolio_buy`）として実装し、内部で `buy_sync`（`pub(crate)`）をトランザクション内で呼び出す。

#### Scenario: 初回の買いで新規ポジション作成
- **WHEN** `db.portfolio_buy("7203", 100, 2000, Some("test"))` を呼び出した場合
- **THEN** quantity=100, avg_cost=2000 の新規ポジションを作成する

#### Scenario: 買い増しで平均コストを更新
- **WHEN** 100株を2000円で購入後、さらに100株を2200円で購入した場合
- **THEN** ポジションを quantity=200, avg_cost=2100（加重平均）に更新する

### Requirement: 売り取引の記録
システムは SHALL 売り取引を記録し、P&L を計算してポジションを更新する。売り操作は `SqliteClient` のメソッド（`portfolio_sell`）として実装し、内部で `sell_sync`（`pub(crate)`）をトランザクション内で呼び出す。

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
システムは SHALL アクティブなポートフォリオポジションの一覧を `DbClient` trait の `list_positions()` メソッドで提供する。

#### Scenario: ポジション出力
- **WHEN** `db.list_positions()` を呼び出した場合
- **THEN** アクティブなポジションを返す（ticker, name, quantity, avg_cost, unrealized_pnl）

### Requirement: ポートフォリオサマリー
システムは SHALL ポートフォリオの集約サマリーを `DbClient` trait の `portfolio_summary()` メソッドで提供する。

#### Scenario: サマリーの算出
- **WHEN** `db.portfolio_summary()` を呼び出した場合
- **THEN** position_count, total_invested, total_current_value, total_unrealized_pnl, total_unrealized_pnl_pct を返す

### Requirement: 取引履歴
システムは SHALL 過去の取引一覧を `DbClient` trait の `trade_history()` メソッドで提供する。

#### Scenario: 件数制限付き取引履歴
- **WHEN** `db.trade_history(20)` を呼び出した場合
- **THEN** 直近20件の取引を返す（ticker, side, date, quantity, price, pnl）

### Requirement: 取引集計の精度保証
システムは SHALL 取引集計（trade_cash_summary）を Rust 側の Decimal 演算で行い、浮動小数点の精度損失を防ぐ。

#### Scenario: 買い総額の集計
- **WHEN** 複数の買い取引が存在する場合
- **THEN** 各取引の price * quantity を Decimal で乗算・加算し、f64 に変換して返す

#### Scenario: 売り総額の集計
- **WHEN** 複数の売り取引が存在する場合
- **THEN** 各取引の price * quantity を Decimal で乗算・加算し、f64 に変換して返す

### Requirement: show summary に実残高セクションを追加
システムは SHALL `kabu show summary` の出力に、立花証券口座から最後に同期した実残高（`account_balance` の最新スナップショット）を表示する。

#### Scenario: 同期済みの場合の表示
- **WHEN** `account_balance` テーブルにレコードがある状態で `kabu show summary` を実行した場合
- **THEN** 「Cash Available」（実残高）と同期日時（synced_at）を併記して表示する

#### Scenario: 未同期の場合の表示
- **WHEN** `account_balance` テーブルが空の状態で `kabu show summary` を実行した場合
- **THEN** 「Cash Available: not synced (run `kabu sync`)」と表示する
