## Why

現状、kabu の預かり金は spec の `initial_cash` から計算上の残高を推定しているのみで、立花証券口座の実残高・実ポジションと連携していない。配当・手数料・税金・口座への入出金が DB に反映されないため、長期運用すると DB 上の推定値と実残高が乖離していく。さらに、想定外の手動取引や API 障害による不整合も検知できない。これは Issue #27（リコンシリエーション）への対応でもある。

## What Changes

- 新コマンド `kabu sync` を追加（立花証券口座と DB を突合）
- 立花証券 API の以下エンドポイントを呼び出して取得：
  - `CLMZanKaiKanougaku` — 買付可能額
  - `CLMGenbutuKabuList` — 現物保有銘柄一覧
- DB 上の `portfolio_positions` と証券口座の建玉を比較し、不整合を検出
- 預かり金の実残高を新規テーブル `account_balance` に記録（履歴保持）
- 不整合検出時はエラーログを出力し、`--fix` フラグで DB を実残高に合わせる
- `show summary` の表示に「実残高」「実ポジション」「乖離」セクションを追加

## Capabilities

### New Capabilities
- `account-sync`: 立花証券口座と DB のリコンシリエーション機能（買付可能額取得、ポジション突合、不整合検知、自動補正）

### Modified Capabilities
- `tachibana-api`: 残高・建玉照会 API（`CLMZanKaiKanougaku`, `CLMGenbutuKabuList`）の `BrokerClient` trait メソッド追加
- `database`: `account_balance` テーブル追加（残高履歴）
- `portfolio`: `show summary` で実残高との比較表示

## Impact

- `src/cmd/sync.rs` — 新規ファイル（sync コマンドハンドラ）
- `src/cmd/mod.rs` — `pub mod sync` 追加
- `src/main.rs` — `Sync` サブコマンドのルーティング
- `src/tachibana/mod.rs` — `BrokerClient` trait に `query_balance()`, `query_positions()` 追加
- `src/tachibana/order.rs` — `CLMZanKaiKanougaku`, `CLMGenbutuKabuList` のリクエスト/レスポンス
- `src/db/mod.rs` — `account_balance` テーブルの CRUD、`save_balance_snapshot()`, `get_latest_balance()` メソッド
- `migrations/V2__add_account_balance.sql` — 新規マイグレーション
- `src/cmd/show.rs` — summary 表示の拡張
- `tests/sync_test.rs` — 新規（MockBrokerClient で動作検証）
