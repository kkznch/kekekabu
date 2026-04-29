## 1. DB マイグレーションと型定義

- [x] 1.1 `migrations/V2__add_account_balance.sql` を作成（id, cash_available, synced_at）
- [x] 1.2 `db::AccountBalance` 構造体を `db/mod.rs` に追加（Serialize, Deserialize）
- [x] 1.3 `DbClient` trait に `save_balance_snapshot()` と `get_latest_balance()` メソッド追加
- [x] 1.4 `SqliteClient` で上記メソッドを実装

## 2. 立花証券 API の照会機能追加

- [x] 2.1 `BrokerBalance` 構造体を `tachibana/order.rs` に定義（cash_available: Decimal）
- [x] 2.2 `BrokerPosition` 構造体を `tachibana/order.rs` に定義（ticker, quantity, avg_cost）
- [x] 2.3 `BrokerClient` trait に `query_balance()` と `query_positions()` メソッド追加
- [x] 2.4 `TachibanaClient::query_balance()` を実装（CLMZanKaiKanougaku の REQUEST/レスポンス）
- [x] 2.5 `TachibanaClient::query_positions()` を実装（CLMGenbutuKabuList の REQUEST/レスポンス）
- [x] 2.6 `parse_balance_response()` のユニットテスト追加
- [x] 2.7 `parse_positions_response()` のユニットテスト追加

## 3. sync コマンド実装

- [x] 3.1 `src/cmd/sync.rs` を新規作成
- [x] 3.2 `cmd::mod.rs` に `pub mod sync` 追加
- [x] 3.3 `Sync { fix: bool }` サブコマンドを `main.rs` の `Command` enum に追加
- [x] 3.4 `cmd::sync::run()` ハンドラ実装（DB + BrokerClient を受け取る DI）
- [x] 3.5 `query_balance()` の結果を `account_balance` に保存
- [x] 3.6 `query_positions()` の結果と DB の `portfolio_positions` を突合
- [x] 3.7 不整合検出時に warn ログ + 構造化レポート出力（`SyncResult`）
- [x] 3.8 `--fix` 指定時のみ DB を補正（quantity を実数に合わせる）

## 4. main.rs と show コマンド統合

- [x] 4.1 `main.rs` で sync 実行時に `TachibanaClient` を作成して DI
- [x] 4.2 `show summary` で `get_latest_balance()` を取得して表示
- [x] 4.3 推定残高と実残高の乖離を計算して表示

## 5. テストと検証

- [x] 5.1 `tests/sync_test.rs` を新規作成（MockBrokerClient で検証）
- [x] 5.2 シナリオ: 不整合なし → DB 変更なし
- [x] 5.3 シナリオ: DB のみに存在するポジション → --fix で DELETE
- [x] 5.4 シナリオ: 実建玉のみに存在するポジション → --fix で INSERT
- [x] 5.5 シナリオ: 数量乖離 → --fix で UPDATE
- [x] 5.6 シナリオ: --fix なしの場合は DB 変更なし
- [x] 5.7 `just ci` で全テスト通過確認

## 6. ドキュメント

- [x] 6.1 README.md に `kabu sync` の使い方を追記
- [x] 6.2 CLAUDE.md の Architecture セクションに sync 追加
