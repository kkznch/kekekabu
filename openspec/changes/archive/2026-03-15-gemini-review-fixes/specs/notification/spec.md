## Purpose

プラットフォーム非依存の通知抽象化基盤。Notifier trait で通知バックエンド（LINE、Slack、ntfy.sh 等）を差し替え可能にし、execute 結果のフォーマット関数を提供する。具体的なバックエンド実装は将来追加。

## Requirements

### Requirement: Notifier trait による通知抽象化
システムは SHALL `Notifier` trait を定義し、`async fn send(&self, message: &str) -> Result<()>` メソッドで通知送信を抽象化する。

#### Scenario: NullNotifier によるメッセージ破棄
- **WHEN** `NullNotifier` に対して `send()` を呼び出した場合
- **THEN** エラーなく Ok(()) を返し、メッセージはどこにも送信されない

### Requirement: execute 結果のフォーマット
システムは SHALL `format_execute_summary()` 関数で `ExecuteResult` をプレーンテキストに変換する。注文・約定・サーキットブレーカー・ストップロスのイベントを含む。

#### Scenario: イベントなしの場合
- **WHEN** `ExecuteResult` に注文・約定・サーキットブレーカー・ストップロスのいずれも含まれない場合
- **THEN** `None` を返す

#### Scenario: 注文ありの場合
- **WHEN** `ExecuteResult` に注文結果が含まれる場合
- **THEN** `[kabu execute]` ヘッダー付きのテキストを返し、`[BUY]` または `[SELL]` で注文内容を表示する

#### Scenario: ストップロス発動の場合
- **WHEN** `ExecuteResult` に `HardStopLossAction` が含まれる場合
- **THEN** `[STOP-LOSS]` プレフィックス付きで銘柄・損失率・閾値を表示する
