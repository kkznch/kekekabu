## Purpose

WebSocket 常駐による約定通知のリアルタイム受信と DB 記録。`kabu watch` コマンドで EVENT I/F に常時接続し、約定イベントを即座に DB に反映する。

## Requirements

### Requirement: WebSocket 常駐接続
システムは SHALL `kabu watch` コマンドで立花証券 EVENT I/F（WebSocket）に常駐接続し、EC（約定通知）イベントをリアルタイムで受信する。

#### Scenario: 常駐接続の開始
- **WHEN** `kabu watch` を実行した場合
- **THEN** 立花証券 API にログインし、sUrlEventWebSocket に WebSocket 接続して EC イベントをサブスクライブし、foreground で待機する

#### Scenario: Ctrl-C による終了
- **WHEN** 常駐中に SIGINT（Ctrl-C）を受信した場合
- **THEN** WebSocket を切断し、立花証券 API からログアウトして正常終了する

### Requirement: 約定通知の DB 記録
システムは SHALL EC イベント受信時に、orders テーブルのステータス更新と portfolio_positions / trades の更新を行う。

#### Scenario: 全部約定通知
- **WHEN** sOrderStatusCode="10" の約定通知を受信した場合
- **THEN** orders テーブルを filled に更新し、portfolio_positions と trades に約定を記録する

#### Scenario: 一部約定通知
- **WHEN** sOrderStatusCode="9" の約定通知を受信した場合
- **THEN** orders テーブルを partial に更新し、約定済み分を portfolio_positions と trades に記録する

#### Scenario: 既に処理済みの注文
- **WHEN** DB 上で既に filled の注文に対する約定通知を受信した場合
- **THEN** 二重処理を防止し、スキップする

### Requirement: 再接続ロジック
システムは SHALL WebSocket 切断時に指数バックオフで再接続を試みる。

#### Scenario: 一時的な切断
- **WHEN** WebSocket 接続が切断された場合
- **THEN** 1s, 2s, 4s, ... 最大 60s の指数バックオフで再接続を試みる。再接続時はログインからやり直す

#### Scenario: 最大リトライ
- **WHEN** 連続して再接続に失敗し続ける場合
- **THEN** 60s 間隔で再接続を試み続ける（プロセスは終了しない）
