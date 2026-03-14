## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: 取引集計の精度保証
システムは SHALL 取引集計（trade_cash_summary）を Rust 側の Decimal 演算で行い、浮動小数点の精度損失を防ぐ。

#### Scenario: 買い総額の集計
- **WHEN** 複数の買い取引が存在する場合
- **THEN** 各取引の price * quantity を Decimal で乗算・加算し、f64 に変換して返す

#### Scenario: 売り総額の集計
- **WHEN** 複数の売り取引が存在する場合
- **THEN** 各取引の price * quantity を Decimal で乗算・加算し、f64 に変換して返す
