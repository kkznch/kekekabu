## Purpose

LLM プロンプト/レスポンスの永続化ログ。判断根拠の事後検証・デバッグ・分析を可能にする。

## Requirements

### Requirement: LLM 呼び出しログの永続化
システムは SHALL LLM バックエンドへの呼び出しごとに、コマンド名・銘柄・バックエンド名・モデル名・temperature・プロンプト全文・レスポンス全文を `llm_logs` テーブルに保存する。

#### Scenario: eval 呼び出しのログ保存
- **WHEN** eval コマンドが LLM バックエンドにプロンプトを送信し応答を受信した場合
- **THEN** command="eval"、対象銘柄の ticker、バックエンド名、モデル名、temperature、プロンプト全文、レスポンス全文を `llm_logs` に保存する

#### Scenario: fetch 呼び出しのログ保存
- **WHEN** fetch コマンドが LLM バックエンドにプロンプトを送信し応答を受信した場合
- **THEN** command="fetch"、対象銘柄の ticker、バックエンド名、プロンプト全文、レスポンス全文を `llm_logs` に保存する

#### Scenario: discover 呼び出しのログ保存
- **WHEN** discover コマンドが LLM バックエンドにプロンプトを送信し応答を受信した場合
- **THEN** command="discover"、ticker=NULL、バックエンド名、プロンプト全文、レスポンス全文を `llm_logs` に保存する

### Requirement: LLM ログの閲覧
システムは SHALL `kabu show llm-logs` コマンドで保存されたログを閲覧できる。

#### Scenario: デフォルトのログ一覧表示
- **WHEN** `kabu show llm-logs` を実行した場合
- **THEN** 直近 20 件のログを新しい順に表示する（id, command, ticker, backend, model, temperature, created_at）

#### Scenario: 件数指定のログ一覧表示
- **WHEN** `kabu show llm-logs --limit 50` を実行した場合
- **THEN** 直近 50 件のログを表示する

#### Scenario: 銘柄フィルタによるログ一覧表示
- **WHEN** `kabu show llm-logs --ticker 7203` を実行した場合
- **THEN** ticker="7203" のログのみを表示する
