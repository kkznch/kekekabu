## MODIFIED Requirements

### Requirement: EVENT I/F WebSocket 接続
システムは SHALL 立花証券 EVENT I/F WebSocket に URL クエリパラメータ方式で接続する。サブスクリプションは接続 URL に `p_rid`, `p_board_no`, `p_eno`, `p_evt_cmd` を指定する。通知データは独自バイナリフォーマット（`^A`=0x01 フィールド区切り、`^B`=0x02 キー/値区切り、`^C`=0x03 値内区切り）である。REQUEST I/F の compress/uncompress は EVENT I/F には適用しない。

#### Scenario: WebSocket URL 構築
- **WHEN** EVENT I/F WebSocket に接続する場合
- **THEN** ログインレスポンスの `sUrlEventWebSocket` に `?p_rid=0&p_board_no=1000&p_eno=0&p_evt_cmd=EC` を付与した URL に接続する

#### Scenario: 独自フォーマットの通知受信
- **WHEN** WebSocket からテキストフレームを受信した場合
- **THEN** `0x01` でフィールド分割、各フィールドを `0x02` でキー/値に分割し、HashMap として返す
