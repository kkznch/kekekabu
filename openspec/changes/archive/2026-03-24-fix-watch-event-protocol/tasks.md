## 1. EVENT I/F パーサー

- [x] 1.1 `event.rs` に `parse_event_fields(text: &str) -> HashMap<String, String>` を追加（`0x01`/`0x02` 分割）
- [x] 1.2 `EventMessage` enum を定義（EC, KP, ST, Other）
- [x] 1.3 `parse_event_message(text: &str) -> EventMessage` を実装（`p_cmd` で分岐）
- [x] 1.4 `ExecutionEvent` 構造体を定義（p_ON, p_NT, p_EXST, p_EXPR, p_EXSR, p_IC, p_BBKB）

## 2. WebSocket 接続方式の修正

- [x] 2.1 `build_event_ws_url(base_url: &str) -> String` を `event.rs` に追加
- [x] 2.2 `watch.rs` で接続 URL にクエリパラメータを付与
- [x] 2.3 JSON サブスクリプション送信を削除（`build_event_subscribe_json` と compress 関連を除去）

## 3. 約定処理の修正

- [x] 3.1 `watch.rs` の受信ループを `parse_event_message` ベースに書き換え
- [x] 3.2 EC イベント（p_NT=12）で `process_fill` を呼び出し、p_EXST で partial/filled を判定
- [x] 3.3 KP イベントで trace ログ出力
- [x] 3.4 ST イベントでエラーログ出力

## 4. テスト

- [x] 4.1 `parse_event_fields` のユニットテスト（正常系、空フィールド、不正入力）
- [x] 4.2 `parse_event_message` のユニットテスト（EC, KP, ST, Other）
- [x] 4.3 `build_event_ws_url` のユニットテスト
- [x] 4.4 `just ci` で全テスト通過確認
