## Why

`kabu watch` の EVENT I/F WebSocket 通信が JSON を送信しているが、立花証券 API の EVENT I/F は独自バイナリフォーマット（`^A`/`^B`/`^C` 区切り）を使用する。このためサーバーが `parameter error` (p_errno=-1) を返し、即座に切断される。接続方式もサブスクリプション方式（メッセージ送信）ではなく、URL クエリパラメータで指定する方式。

## What Changes

- WebSocket 接続 URL にクエリパラメータを付与（`?p_rid=0&p_board_no=1000&p_eno=0&p_evt_cmd=EC`）
- JSON サブスクリプションメッセージの送信を削除
- 受信データのパーサーを独自フォーマット（`0x01`/`0x02` 区切り）に変更
- EC 通知の約定判定を `p_EXST`（0=未約定、1=一部約定、2=全部約定）に変更
- `build_event_subscribe_json()` を削除し、URL 構築関数に置換
- KP（キープアライブ）イベントを認識してログ出力

## Capabilities

### New Capabilities

### Modified Capabilities
- `watch`: EVENT I/F の通信プロトコルを JSON から独自バイナリフォーマットに修正
- `tachibana-api`: EVENT I/F のデータフォーマット仕様を追記

## Impact

- `src/cmd/watch.rs` — WebSocket 接続方式と受信パーサーの全面書き換え
- `src/tachibana/event.rs` — `build_event_subscribe_json()` 削除、`build_event_ws_url()` 新設、`parse_event_message()` 新設
- `src/tachibana/compress.rs` — EVENT I/F では compress/uncompress 不要（削除ではなく使用箇所から除去）
