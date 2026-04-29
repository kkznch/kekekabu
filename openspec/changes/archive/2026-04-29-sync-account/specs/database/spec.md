## ADDED Requirements

### Requirement: account_balance テーブルで残高履歴を保持
システムは SHALL `account_balance` テーブルを新設し、立花証券口座から取得した買付可能額のスナップショットを時系列で記録する。

#### Scenario: 残高スナップショット保存
- **WHEN** `kabu sync` が立花証券 API から買付可能額を取得した場合
- **THEN** `account_balance` テーブルに `(cash_available, synced_at)` の新規行を INSERT する。既存行は更新しない（履歴保持）

#### Scenario: 最新残高の取得
- **WHEN** `DbClient::get_latest_balance()` を呼び出した場合
- **THEN** `account_balance` テーブルから `synced_at` の最新行を返す。レコードが存在しない場合は `None` を返す

#### Scenario: refinery V2 マイグレーションで作成
- **WHEN** `kabu db migrate` を実行した場合
- **THEN** `migrations/V2__add_account_balance.sql` が適用され、`account_balance` テーブルが作成される
