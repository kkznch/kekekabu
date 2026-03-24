## Context

`kabu watch` は WebSocket で立花証券 EVENT I/F に接続し約定通知を受信する。現在の実装は JSON でサブスクリプションメッセージを送信し、JSON レスポンスをパースしているが、EVENT I/F は独自バイナリフォーマットを使用する。

公式仕様（`api_event_if_v4r7.pdf`）によると：
- サブスクリプションは **URL クエリパラメータ** で行う（メッセージ送信不要）
- 通知データは `^A`(0x01) / `^B`(0x02) / `^C`(0x03) 区切りの独自フォーマット
- ShiftJIS テキストは WebSocket 版では BASE64 エンコード
- REQUEST I/F の compress/uncompress は EVENT I/F には適用されない

## Goals / Non-Goals

**Goals:**
- EVENT I/F 独自フォーマットでの正常な接続と通知受信
- EC（約定通知）の正確なパース（p_NT, p_EXST, p_EXPR, p_EXSR）
- KP（キープアライブ）の認識

**Non-Goals:**
- FD（時価情報配信）のサポート
- NS（ニュース配信）のサポート
- REQUEST I/F の GET→POST 移行（別 change）

## Decisions

### Decision 1: URL クエリパラメータでサブスクリプション

`sUrlEventWebSocket` + `?p_rid=0&p_board_no=1000&p_eno=0&p_evt_cmd=EC` の形式で接続。`p_evt_cmd=EC` で約定通知のみ受信。

### Decision 2: イベントパーサーを event.rs に新設

`parse_event_message(text: &str) -> EventMessage` を新設。`0x01`/`0x02` で分割し、`p_cmd` で分岐する enum を返す。

```rust
enum EventMessage {
    EC(ExecutionEvent),  // 約定通知
    KP,                  // キープアライブ
    ST(ErrorStatus),     // エラー
    Other(String),       // 未対応イベント
}
```

### Decision 3: 約定判定は p_NT と p_EXST を併用

- `p_NT=12`（約定成立）のときのみ DB に記録
- `p_EXST=1` → partial fill、`p_EXST=2` → full fill
- `p_EXPR`（約定値段）と `p_EXSR`（約定数量）を使用

### Decision 4: compress/uncompress は watch から削除

EVENT I/F は独自フォーマットのため、REQUEST I/F 用の compress/uncompress は不要。watch.rs から compress の import と使用を全て除去。

## Risks / Trade-offs

- [Risk] BASE64 エンコードされた ShiftJIS テキスト（銘柄名等）のデコードが必要 → `p_IN` のみ該当、DB には不要なので初期実装ではスキップ
- [Risk] p_eno（再送開始位置）の管理が必要 → 初期実装では 0（全件）で接続、将来的には最後に受信した p_ENO を保存して再接続時に指定
