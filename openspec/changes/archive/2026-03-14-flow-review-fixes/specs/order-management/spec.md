## MODIFIED Requirements

### Requirement: 注文の永続化
システムは SHALL 発注した注文を orders テーブルに保存し、ステータス遷移を追跡する。

#### Scenario: 新規注文の記録
- **WHEN** 指値注文を発注した場合
- **THEN** orders テーブルに stock_id, side, order_type, price, quantity, status="pending", tachibana_order_id, request_id, evaluation_id を保存する

#### Scenario: 一部約定による更新
- **WHEN** pending 状態の注文が一部約定した場合
- **THEN** orders テーブルの status を "partial" に更新し、約定済み分の filled_price, filled_quantity, filled_at を記録する

#### Scenario: 全部約定による更新
- **WHEN** pending または partial 状態の注文が全部約定した場合
- **THEN** orders テーブルの status を "filled" に更新し、filled_price, filled_quantity, filled_at を記録する

#### Scenario: 失効による更新
- **WHEN** pending 状態の注文が失効した場合
- **THEN** orders テーブルの status を "expired" に更新する

#### Scenario: 拒否による更新
- **WHEN** 注文が受付エラーで拒否された場合
- **THEN** orders テーブルの status を "rejected" に更新する

### Requirement: 未決済注文の一覧取得
システムは SHALL status="pending" または status="partial" の未決済注文を一覧取得できる。

#### Scenario: settle 対象の取得
- **WHEN** settle 処理の開始時
- **THEN** status が "pending" または "partial" の全注文を tachibana_order_id, 営業日とともに返す
