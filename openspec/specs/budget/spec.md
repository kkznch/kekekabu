## Purpose

投資資金管理。立花証券から `kabu sync` で取得した実残高（`account_balance.cash_available`）を使用し、LLM プロンプトに Budget Context として注入する。`kabu sync` の実行を前提とし、初期資金の手動定義（`initial_cash`）は廃止された。

## Requirements

### Requirement: 同期済み実残高を Budget Context のソースとする
システムは SHALL `account_balance` テーブルの最新スナップショット（`get_latest_balance()`）から `cash_available` を取得し、Budget Context の生成に使用する。

#### Scenario: 同期済み残高がある場合
- **WHEN** `account_balance` テーブルに残高スナップショットがある場合
- **THEN** 最新の `cash_available` を取得して Budget Context を生成する

#### Scenario: 残高未同期の場合（discover/eval/workflow）
- **WHEN** `account_balance` テーブルが空の状態で discover / eval / workflow run を実行した場合
- **THEN** 「Run `kabu sync` first」のエラーメッセージで異常終了する

### Requirement: Budget Context をプロンプト用テキストとして生成する
システムは SHALL Budget Context を LLM プロンプトに注入可能なテキストとして生成し、Cash Available（実残高）、同期日時、Active Positions（保有銘柄数）を含める。

#### Scenario: Budget Context の生成
- **WHEN** `cash_available = 210000`, `position_count = 2`, `synced_at = "2026-04-29 10:00:00"` で `build_budget_context()` を呼び出した場合
- **THEN** 「Cash Available: 210000 JPY (broker-synced at 2026-04-29 10:00:00)」「Active Positions: 2」を含むテキストを返す
