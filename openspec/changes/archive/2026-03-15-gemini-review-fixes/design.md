## Context

Gemini レビューで指摘されたリスク（LLM 依存の損切り、テスト DB 乖離、通知欠如）への対応。既存の execute パイプライン（8 フェーズ構成）に安全機構を挿入し、テスト基盤を改善する。

## Goals / Non-Goals

**Goals:**
- spec の `[execution].stop_loss` を読み取り、LLM 判断とは独立して強制売りを実行する
- spec の `[execution].max_position_size` × `[budget].initial_cash` を超える買い注文を reject する
- テスト用 `open_in_memory()` で refinery マイグレーションを適用する
- 通知の抽象化基盤（Notifier trait）を用意する

**Non-Goals:**
- 通知の具体的なバックエンド実装（LINE, Slack, ntfy.sh 等）
- リコンシリエーション（証券口座との突合）機能
- トレーリングストップの実装

## Decisions

### Decision 1: execute のフェーズ構成に stop-loss を挿入

Circuit breaker（Phase 2）の後、eval 処理（Phase 4）の前にハードストップロス判定（Phase 3）を挿入する。stop-loss でトリガーされた売りは eval の Sell 判断とは独立して処理される。

Phase 構成: Settle(1) → Circuit Breaker(2) → **Hard Stop-Loss(3)** → Eval Signals(4) → Inject Stop-Loss Signals(5) → Place Orders(6) → WebSocket Wait(7) → Logout(8)

### Decision 2: stop-loss は成行注文、eval は指値注文

stop-loss は緊急性が高いため成行注文（`force_market: true`）。通常の eval ベースの売買は従来通り指値注文。Signal 構造体に `force_market` フィールドを追加して区別する。

### Decision 3: stop-loss と eval の sell が競合した場合は eval 側を優先

同一銘柄に対して eval の Sell と stop-loss の強制売りが同時に発生した場合、eval の Sell シグナルが既に存在するなら stop-loss の注入をスキップ。逆に stop-loss 対象銘柄への Buy シグナルはブロックする。

### Decision 4: InvestmentSpec に個別アクセサを追加（汎用パーサーではなく）

`execution_stop_loss()` / `execution_max_position_size()` を個別メソッドとして追加。内部で共通の `execution_float()` ヘルパーを使う。汎用的な `get_float("execution.stop_loss")` ではなく、型安全な個別アクセサにすることで誤用を防ぐ。

### Decision 5: テスト DB は refinery::embed_migrations! を使用

`open_in_memory()` で `include_str!("V1__initial_schema.sql")` の直接実行をやめ、`embedded::migrations::runner().run(conn)` を使用。V2 以降のマイグレーション追加時にテスト側の手動更新が不要になる。

### Decision 6: Notifier trait は DbClient パターンに準拠

`async_trait` を使った trait + ファクトリ関数パターン。現時点では NullNotifier のみ実装し、config や main.rs への接続は具体バックエンド実装時に行う。

## Risks / Trade-offs

- [Risk] stop_loss が spec に未定義の場合 → Option<f64> で None ならスキップ。安全側に倒す（チェックしない ≠ 売らない）
- [Risk] 成行注文の約定価格が想定外 → 成行は stop-loss の緊急性を優先した判断。価格ギャップは許容する
- [Trade-off] Notifier は trait だけで具体実装なし → 基盤が先にあることで、バックエンド追加時のコード変更が最小になる
