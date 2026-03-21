## MODIFIED Requirements

### Requirement: サーキットブレーカーによる異常検知
システムは SHALL サーキットブレーカーの価格変動チェックにおいて、全価格履歴ではなく直近2件の終値のみを取得して判定する。`DbClient` trait に `get_latest_closes(stock_id, n)` メソッドを追加し、circuit_breaker はこれを使用する。

#### Scenario: 直近終値による異常判定
- **WHEN** circuit_breaker が銘柄の価格変動をチェックする場合
- **THEN** `get_latest_closes(stock_id, 2)` で直近2件の終値を取得し、変動率を算出する。全価格履歴（fetch_price_data）は使用しない

#### Scenario: 価格データ不足
- **WHEN** 直近終値が2件未満の場合
- **THEN** その銘柄の異常判定をスキップする（従来と同じ挙動）
