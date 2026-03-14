## 1. InvestmentSpec の execution アクセサ

- [x] 1.1 `execution_float()` 共通ヘルパーメソッドを追加
- [x] 1.2 `execution_stop_loss()` メソッドを追加
- [x] 1.3 `execution_max_position_size()` メソッドを追加
- [x] 1.4 テスト追加（stop_loss / max_position_size / params_absent）

## 2. execute コマンドのハードストップロス

- [x] 2.1 `HardStopLossAction` 構造体を `ExecuteResult` に追加
- [x] 2.2 `Signal` に `force_market` フィールドを追加
- [x] 2.3 `execute::run()` のシグネチャに `&InvestmentSpec` を追加
- [x] 2.4 Phase 3: ハードストップロス判定ロジックを実装
- [x] 2.5 Phase 5: stop-loss 強制売りシグナルの注入（eval との競合制御含む）
- [x] 2.6 stop-loss 対象銘柄への Buy シグナルブロック

## 3. execute コマンドの最大エクスポージャーチェック

- [x] 3.1 Phase 6: 買い注文前に `max_position_size × initial_cash` チェックを追加
- [x] 3.2 超過時は reject して `order_results` に記録

## 4. main.rs の接続

- [x] 4.1 execute 呼び出し時に `spec::load_spec()` で InvestmentSpec を読み込み渡す

## 5. テスト DB の refinery 移行

- [x] 5.1 `open_in_memory()` を `include_str!` から `embedded::migrations::runner()` に変更

## 6. 通知基盤

- [x] 6.1 `notification.rs` に `Notifier` trait と `NullNotifier` を定義
- [x] 6.2 `format_execute_summary()` 関数を実装
- [x] 6.3 テスト追加（empty / with_order / with_stop_loss / circuit_breaker / null_notifier）

## 7. CLAUDE.md 更新

- [x] 7.1 Architecture セクションに `notification.rs` と execute の安全機構を反映

## 8. 検証

- [x] 8.1 `just ci` で全テスト通過を確認
