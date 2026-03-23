## Why

現在の約定通知受信は `execute` コマンド内でタイムアウト付き WebSocket 接続を行う方式。注文後にタイムアウトすると約定を見逃し、次回 execute の settle フェーズまで検知できない。常駐型の `kabu watch` コマンドで WebSocket を維持し、約定通知をリアルタイムで DB に記録する。

## What Changes

- `kabu watch` コマンドの新設（WebSocket 常駐、約定通知を DB に書き込み）
- `execute` から WebSocket 待受ロジック（Phase 7）を削除し、`watch` に移管
- `execute` は注文発注後に即座に return し、約定検知は `watch` または次回 `settle` に委ねる
- 通知機能との統合（`Notifier` trait で約定通知を送信）

## Capabilities

### New Capabilities
- `watch`: WebSocket 常駐による約定通知のリアルタイム受信と DB 記録

### Modified Capabilities
- `trade-execution`: execute から WebSocket Phase 7 を削除

## Impact

- `src/cmd/watch.rs` — 新規ファイル
- `src/cmd/mod.rs` — `pub mod watch` 追加
- `src/cmd/execute.rs` — Phase 7（WebSocket fill wait）を削除
- `src/main.rs` — `Watch` コマンドバリアント追加
- `src/tachibana/event.rs` — 常駐接続用のロジック追加（再接続、heartbeat）
