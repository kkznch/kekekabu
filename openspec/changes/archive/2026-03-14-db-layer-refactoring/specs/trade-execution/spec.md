## MODIFIED Requirements

### Requirement: settle フェーズによる前回注文の約定確認
システムは SHALL execute 冒頭で orders テーブルの未決済注文（pending または partial）を立花証券 API で照会し、約定済みなら `DbClient` の `update_order_and_record_fill` メソッドで portfolio に記録する。`FillParams` 構造体は `db` モジュールで定義する。

#### Scenario: 約定済み注文の settle
- **WHEN** 未決済注文の sOrderStatusCode が "10"（全部約定）の場合
- **THEN** `conn.update_order_and_record_fill(FillParams { ... })` を呼び出してポジション・取引履歴を更新し、orders.status を "filled" に更新する

#### Scenario: 一部約定注文の settle
- **WHEN** 未決済注文の sOrderStatusCode が "9"（一部約定）の場合
- **THEN** 約定済み分を `update_order_and_record_fill` で portfolio に記録し、orders.status を "partial" に更新する。残りは次回の settle で再確認する

#### Scenario: 失効注文の settle
- **WHEN** 未決済注文の sOrderStatusCode が "12"（全部失効）の場合
- **THEN** `conn.update_order_status()` で orders.status を "expired" に更新し、portfolio は変更しない

#### Scenario: まだ未約定の注文
- **WHEN** 未決済注文の sOrderStatusCode が "1"（未約定）の場合
- **THEN** orders.status は "pending" のまま残し、次回の settle で再確認する

#### Scenario: settle 対象がない場合
- **WHEN** 未決済注文（pending/partial）が存在しない場合
- **THEN** 立花 API にログインせず settle フェーズをスキップする
