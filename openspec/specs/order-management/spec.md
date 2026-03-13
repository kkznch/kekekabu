## Purpose

注文ライフサイクル管理。orders テーブルによる注文追跡、ステータス遷移、べき等性の request_id による重複防止を提供する。

## Requirements

### Requirement: 注文の永続化
システムは SHALL 発注した注文を orders テーブルに保存し、ステータス遷移を追跡する。

#### Scenario: 新規注文の記録
- **WHEN** 指値注文を発注した場合
- **THEN** orders テーブルに stock_id, side, order_type, price, quantity, status="pending", tachibana_order_id, request_id, evaluation_id を保存する

#### Scenario: 約定による更新
- **WHEN** pending 状態の注文が全部約定した場合
- **THEN** orders テーブルの status を "filled" に更新し、filled_price, filled_quantity, filled_at を記録する

#### Scenario: 失効による更新
- **WHEN** pending 状態の注文が失効した場合
- **THEN** orders テーブルの status を "expired" に更新する

#### Scenario: 拒否による更新
- **WHEN** 注文が受付エラーで拒否された場合
- **THEN** orders テーブルの status を "rejected" に更新する

### Requirement: べき等性の保証
システムは SHALL request_id（UNIQUE 制約）により同一注文の重複発注を防止する。

#### Scenario: 同じ評価からの重複注文防止
- **WHEN** 同日・同銘柄・同方向・同 evaluation_id の注文が既に orders テーブルに存在する場合
- **THEN** 重複注文をスキップし、既存の注文情報を返す

#### Scenario: request_id のフォーマット
- **WHEN** 注文を発注する場合
- **THEN** request_id を `{date}-{ticker}-{side}-{evaluation_id}` 形式で生成する

### Requirement: pending 注文の一覧取得
システムは SHALL status="pending" の注文を一覧取得できる。

#### Scenario: settle 対象の取得
- **WHEN** settle 処理の開始時
- **THEN** status="pending" の全注文を tachibana_order_id, 営業日とともに返す

### Requirement: 注文一覧の表示
システムは SHALL `kabu show orders` コマンドで注文履歴を表示できる。

#### Scenario: デフォルトの注文一覧表示
- **WHEN** `kabu show orders` を実行した場合
- **THEN** 直近 20 件の注文を新しい順に表示する（ticker, side, price, quantity, status, created_at）

#### Scenario: ステータスフィルタによる表示
- **WHEN** `kabu show orders --status pending` を実行した場合
- **THEN** status="pending" の注文のみを表示する
