## ADDED Requirements

### Requirement: show summary に実残高セクションを追加
システムは SHALL `kabu show summary` の出力に、立花証券口座から最後に同期した実残高（`account_balance` の最新スナップショット）を表示する。

#### Scenario: 同期済みの場合の表示
- **WHEN** `account_balance` テーブルにレコードがある状態で `kabu show summary` を実行した場合
- **THEN** 「Cash Available」（実残高）と「Last Synced」（同期日時）を表示する

#### Scenario: 未同期の場合の表示
- **WHEN** `account_balance` テーブルが空の状態で `kabu show summary` を実行した場合
- **THEN** 「Cash Available: not synced (run `kabu sync`)」と表示する

#### Scenario: 推定残高との乖離表示
- **WHEN** spec の `initial_cash` から計算した推定残高と実残高が異なる場合
- **THEN** 「Estimated」「Actual」「Diff」を併記し、乖離額を表示する
