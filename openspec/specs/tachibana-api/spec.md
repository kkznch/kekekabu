## Purpose

立花証券 e支店 API クライアント。認証、REQUEST I/F（注文入力・照会）、EVENT I/F（WebSocket 約定通知）を統合的に提供する。

## Requirements

### Requirement: 立花証券 API 認証
システムは SHALL 立花証券 e支店 API の認証 I/F にログインし、仮想 URL（sUrlRequest, sUrlEvent, sUrlEventWebSocket）を取得してセッションを管理する。

#### Scenario: ログイン成功
- **WHEN** 有効な userId, password, secondPassword で認証 I/F にリクエストした場合
- **THEN** sUrlRequest, sUrlEvent, sUrlEventWebSocket を含むセッション情報を返す

#### Scenario: 認証失敗
- **WHEN** 無効な認証情報でログインした場合
- **THEN** エラーメッセージ付きで認証失敗を返す

#### Scenario: 金商法交付書面未読
- **WHEN** sKinsyouhouMidokuFlg が "1"（未読）の場合
- **THEN** 仮想 URL が発行されない旨のエラーを返し、標準 Web での確認を促す

#### Scenario: ログアウト
- **WHEN** セッション終了時にログアウトを実行した場合
- **THEN** 仮想 URL を無効化する

### Requirement: REQUEST I/F による注文入力
システムは SHALL REQUEST I/F（sUrlRequest）を通じて株式現物の指値注文を発注する。リクエストには p_no（通し番号）と p_sd_date（クライアント時刻）を含める。

#### Scenario: 指値買い注文の発注
- **WHEN** 銘柄コード、指値価格、数量を指定して買い注文を発注した場合
- **THEN** CLMKabuNewOrder で注文を送信し、注文番号（sOrderNumber）を含む応答を返す

#### Scenario: 指値売り注文の発注
- **WHEN** 保有銘柄に対して指値価格と数量を指定して売り注文を発注した場合
- **THEN** CLMKabuNewOrder で売り注文を送信し、注文番号を含む応答を返す

#### Scenario: p_no のインクリメント
- **WHEN** 連続してリクエストを送信する場合
- **THEN** p_no を毎回 +1 以上インクリメントして送信する

#### Scenario: Shift-JIS レスポンスのデコード
- **WHEN** API からレスポンスを受信した場合
- **THEN** Shift-JIS エンコードのレスポンスを UTF-8 に変換してパースする

### Requirement: REQUEST I/F による約定照会
システムは SHALL CLMOrderListDetail コマンドで注文番号と営業日を指定して注文の約定状態を照会する。

#### Scenario: 約定済み注文の照会
- **WHEN** sOrderStatusCode が "10"（全部約定）の注文を照会した場合
- **THEN** sYakuzyouPrice（約定単価）、sYakuzyouSuryou（約定株数）、aYakuzyouSikkouList（約定リスト）を返す

#### Scenario: 一部約定注文の照会
- **WHEN** sOrderStatusCode が "9"（一部約定）の注文を照会した場合
- **THEN** 約定済み分の sYakuzyouPrice、sYakuzyouSuryou を返す。残り未約定分は次回照会で確認する

#### Scenario: 未約定注文の照会
- **WHEN** sOrderStatusCode が "1"（未約定）の注文を照会した場合
- **THEN** 注文情報を返し、約定フィールドは空または未設定

#### Scenario: 失効注文の照会
- **WHEN** sOrderStatusCode が "12"（全部失効）の注文を照会した場合
- **THEN** 失効状態を返す

### Requirement: EVENT I/F WebSocket による約定通知受信
システムは SHALL sUrlEventWebSocket に WebSocket 接続し、EC（約定通知）イベントのサブスクリプションメッセージを送信した上で、約定通知をリアルタイムで受信する。全部約定（status "10"）と一部約定（status "9"）の両方を処理する。

#### Scenario: WebSocket 接続とサブスクリプション
- **WHEN** WebSocket に接続した場合
- **THEN** EC イベントのサブスクリプションメッセージ（p_evt_cmd="EC"）を送信してから約定通知の待機を開始する

#### Scenario: 全部約定通知の受信
- **WHEN** WebSocket で sOrderStatusCode="10" の約定通知を受信した場合
- **THEN** 約定情報（銘柄、約定価格、約定数量）を含む通知として処理する

#### Scenario: 一部約定通知の受信
- **WHEN** WebSocket で sOrderStatusCode="9" の約定通知を受信した場合
- **THEN** 一部約定分の約定情報（約定価格、約定数量）を含む通知として処理する

#### Scenario: タイムアウト切断
- **WHEN** 指定時間内に約定通知が受信されなかった場合
- **THEN** WebSocket 接続を切断し、タイムアウトを返す

#### Scenario: 接続エラー
- **WHEN** WebSocket 接続に失敗した場合
- **THEN** エラーログを出力し、settle フォールバック（REQUEST I/F 照会）で対応する
