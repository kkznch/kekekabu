## 1. Config 拡張

- [x] 1.1 `config.rs` に `[tachibana]` セクション（user_id, password, second_password, event_timeout_secs）を追加
- [x] 1.2 環境変数オーバーライド（TACHIBANA_USER_ID, TACHIBANA_PASSWORD, TACHIBANA_SECOND_PASSWORD）を追加
- [x] 1.3 `config init` のテンプレートに `[tachibana]` セクションを追加
- [x] 1.4 `config validate` で tachibana 設定のバリデーションを追加（execute 非 dry-run 時のみ必須）

## 2. Database — orders テーブル

- [x] 2.1 `db/schema.rs` に CREATE_ORDERS_TABLE を追加し ALL_SCHEMAS に登録
- [x] 2.2 `db/mod.rs` に orders CRUD を追加: save_order, update_order_status, list_pending_orders, list_orders
- [x] 2.3 `db/mod.rs` の table_stats に "orders" を追加
- [x] 2.4 orders の DB テスト追加（save, update status, list pending, request_id 重複テスト）

## 3. 立花証券 API クライアント

- [x] 3.1 `src/tachibana/mod.rs` — TachibanaClient 構造体、ログイン/ログアウト、セッション管理
- [x] 3.2 `src/tachibana/request.rs` — REQUEST I/F ヘルパー（JSON 構築、URL エンコード、p_no 管理、Shift-JIS デコード）
- [x] 3.3 `src/tachibana/order.rs` — CLMKabuNewOrder（注文入力）、CLMOrderListDetail（約定照会）
- [x] 3.4 `src/tachibana/event.rs` — EVENT I/F WebSocket 接続、約定通知パース、タイムアウト制御
- [x] 3.5 `lib.rs` に `pub mod tachibana` を追加
- [x] 3.6 `Cargo.toml` に依存追加（tokio-tungstenite, encoding_rs, url）

## 4. Execute コマンド改修

- [x] 4.1 execute に settle フェーズを追加（pending 注文の約定確認 → portfolio::buy/sell 呼び出し）
- [x] 4.2 シグナル生成にべき等チェックを追加（同じ eval から既存 order があればスキップ）
- [x] 4.3 dry-run でない場合に立花 API 経由で指値注文を発注し orders テーブルに記録
- [x] 4.4 発注後の短時間 WebSocket 約定待ちを追加（タイムアウト設定可能）
- [x] 4.5 ExecuteResult に settle 結果と発注結果のフィールドを追加

## 5. Show コマンド拡張

- [x] 5.1 `kabu show orders` コマンドを追加（--limit, --status オプション）
- [x] 5.2 orders の HumanDisplay 実装

## 6. テスト

- [x] 6.1 config テスト: tachibana セクションの読み込み・環境変数オーバーライド
- [x] 6.2 orders DB テスト: CRUD、べき等性、ステータス遷移
- [x] 6.3 execute テスト: settle ロジック（モック or テストヘルパー）
- [x] 6.4 tachibana request ヘルパーのユニットテスト（JSON 構築、URL エンコード、Shift-JIS デコード）
- [x] 6.5 table_stats テスト更新（10 テーブル）
- [x] 6.6 全テスト pass 確認（`just ci`）

## 7. ドキュメント更新

- [x] 7.1 CLAUDE.md の DB テーブル数・Architecture セクション更新
- [x] 7.2 README.md に execute の実行モード・orders コマンドの説明追加
