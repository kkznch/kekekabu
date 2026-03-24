## MODIFIED Requirements

### Requirement: WebSocket 常駐で約定通知をリアルタイム受信
システムは SHALL `kabu watch` コマンドで立花証券 EVENT I/F WebSocket に常駐接続し、約定通知（EC イベント）をリアルタイムで受信して DB に反映する。接続は URL クエリパラメータ方式でサブスクリプションし、受信データは独自バイナリフォーマット（`^A`/`^B`/`^C` 区切り）をパースする。

#### Scenario: WebSocket 接続と EC サブスクリプション
- **WHEN** `kabu watch` を実行した場合
- **THEN** ログイン後、`sUrlEventWebSocket` に `?p_rid=0&p_board_no=1000&p_eno=0&p_evt_cmd=EC` を付与して WebSocket 接続する。JSON サブスクリプションメッセージは送信しない

#### Scenario: 独自フォーマットのパース
- **WHEN** WebSocket で `p_no^B1^Ap_date^B...^Ap_cmd^BEC^A...` 形式のメッセージを受信した場合
- **THEN** `0x01`（フィールド区切り）と `0x02`（キー/値区切り）でパースし、`p_cmd` でイベント種別を判定する

#### Scenario: 約定成立（p_NT=12）の検知
- **WHEN** EC イベントで `p_NT=12`（約定成立）を受信した場合
- **THEN** `p_EXST` で約定状態を判定し（1=一部約定、2=全部約定）、`p_ON`（注文番号）、`p_EXPR`（約定価格）、`p_EXSR`（約定数量）を DB に記録する

#### Scenario: KP（キープアライブ）の処理
- **WHEN** `p_cmd=KP` のメッセージを受信した場合
- **THEN** trace ログに記録し、接続を維持する（5秒間通知がない場合に送信される生存確認）

#### Scenario: ST（エラー）の処理
- **WHEN** `p_cmd=ST` のメッセージを受信した場合
- **THEN** `p_errno` と `p_err` をログに記録し、サーバー切断後に再接続を試行する
