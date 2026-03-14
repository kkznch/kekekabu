## Why

Gemini によるプロジェクトレビュー（`.keke/2026-03-14_REVIEW_BY_GEMINI.md`）で指摘された4点のうち、リアルマネー投入前に必要な安全機構とテスト基盤の改善を行う。損切りを LLM に依存するリスク、テスト DB とのスキーマ乖離、通知基盤の欠如を解消する。

## What Changes

- `execute` コマンドにルールベースのハードストップロスを追加（LLM 判断とは独立した強制売り）
- `execute` コマンドに最大ポジションサイズチェックを追加（spec の `max_position_size × initial_cash` を超える買い注文を reject）
- `InvestmentSpec` に `execution_stop_loss()` / `execution_max_position_size()` アクセサを追加
- テスト用 `open_in_memory()` を `refinery::embed_migrations!` に移行（V1 SQL 直接実行を廃止）
- `Notifier` trait による通知基盤を追加（具体的バックエンド実装は将来）

## Capabilities

### New Capabilities
- `hard-stop-loss`: LLM 非依存のルールベース強制損切り + 最大エクスポージャーチェック
- `notification`: Notifier trait による通知抽象化基盤

### Modified Capabilities
- `database`: テスト用 `open_in_memory()` で refinery マイグレーションを使用するように変更

## Impact

- `src/spec.rs` — `execution_stop_loss()`, `execution_max_position_size()`, `execution_float()` メソッド追加
- `src/cmd/execute.rs` — ハードストップロス判定フェーズ、max exposure チェック、Signal に `force_market` フィールド追加
- `src/main.rs` — execute 呼び出し時に spec を読み込んで渡す
- `src/db/mod.rs` — `open_in_memory()` を refinery 使用に変更
- `src/notification.rs` — 新規ファイル（Notifier trait, NullNotifier, format_execute_summary）
- `src/lib.rs` — `pub mod notification` 追加
