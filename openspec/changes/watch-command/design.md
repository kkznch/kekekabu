## Context

現在 `execute` の Phase 7 で WebSocket 接続 → タイムアウト付き fill 待ち → 切断を行っている。EVENT I/F は 1 顧客 1 接続だが、REQUEST I/F（注文・照会）とは独立しており、WebSocket を張りながら注文を出すのが API の設計思想。

## Goals / Non-Goals

**Goals:**
- `kabu watch` で WebSocket を常駐させ、約定通知を即座に DB に記録
- execute を注文発注に専念させ、WebSocket 待受を分離
- 将来的な通知機能（Notifier trait）との統合点を用意

**Non-Goals:**
- リアルタイム株価ボード（KP イベント）の受信（将来拡張）
- 複数セッションの同時管理

## Decisions

### Decision 1: watch は独立した常駐プロセスとして動作

`kabu watch` は foreground プロセスとして動作し、Ctrl-C で終了する。launchd や systemd でデーモン化するのはユーザーの選択。内部では login → WebSocket 接続 → EC イベント受信ループ → DB 更新を行う。

### Decision 2: execute から Phase 7 を削除

execute は注文発注後に即座に結果を返す。約定の検知は:
1. `watch` が常駐していれば即時反映
2. `watch` が動いていなくても、次回 `execute` の Phase 1（settle）で ORDER I/F 照会により検知

これにより execute のライフサイクルがシンプルになる。

### Decision 3: 再接続ロジック

WebSocket が切断された場合、指数バックオフ（1s, 2s, 4s, ... 最大 60s）で再接続を試みる。再接続時にはログインからやり直す（仮想 URL が無効化されている可能性があるため）。

### Decision 4: 約定検知時の DB 更新ロジック

watch が EC イベントを受信したら、execute の settle と同じロジック（`update_order_and_record_fill`）で orders テーブルと portfolio_positions, trades を更新する。

## Risks / Trade-offs

- [Risk] watch と execute が同時に同一注文を settle → orders テーブルの idempotent 更新（status check）で二重処理を防止
- [Risk] WebSocket 切断中の約定見逃し → settle フォールバックで次回 execute 時に検知
- [Trade-off] 2 プロセス管理が必要 → launchd/service コマンドで管理可能
