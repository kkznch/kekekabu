## MODIFIED Requirements

### Requirement: 注文照会
システムは SHALL REQUEST I/F を使用して注文の約定状況を照会する。

#### Scenario: 全部約定注文の照会
- **WHEN** 注文照会で status_code="10" が返された場合
- **THEN** filled_price, filled_quantity を含む約定情報を返す

#### Scenario: 一部約定注文の照会
- **WHEN** 注文照会で status_code="9" が返された場合
- **THEN** 一部約定として filled_price, filled_quantity を含む約定情報を返す

### Requirement: EVENT I/F による約定通知
システムは SHALL WebSocket で EVENT I/F に接続し、約定通知をリアルタイムで受信する。

#### Scenario: WebSocket 接続とサブスクリプション
- **WHEN** EVENT I/F に WebSocket 接続を確立した場合
- **THEN** 接続後にサブスクリプションメッセージを送信し、約定通知の受信を開始する

#### Scenario: 全部約定通知の受信
- **WHEN** status_code="10" の約定通知を受信した場合
- **THEN** 約定情報（order_number, filled_price, filled_quantity）をパースして返す

#### Scenario: 一部約定通知の受信
- **WHEN** status_code="9" の約定通知を受信した場合
- **THEN** 一部約定として約定情報をパースして返す
