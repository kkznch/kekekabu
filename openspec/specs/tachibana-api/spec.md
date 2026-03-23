## Purpose

立花証券 e支店 API クライアント。認証、REQUEST I/F（注文入力・照会）、EVENT I/F（WebSocket 約定通知）を統合的に提供する。

## Requirements

### Requirement: 立花証券 API 認証
システムは SHALL 立花証券 e支店 API の認証 I/F に HTTPS POST で圧縮された JSON body を送信してログインし、レスポンスを展開して仮想 URL（sUrlRequest, sUrlEvent, sUrlEventWebSocket）を取得してセッションを管理する。

#### Scenario: ログイン成功（圧縮レスポンス）
- **WHEN** 有効な userId, password, secondPassword で圧縮リクエストを送信した場合
- **THEN** 圧縮されたレスポンスを展開し、sUrlRequest, sUrlEvent, sUrlEventWebSocket を含むセッション情報を返す

#### Scenario: 認証失敗
- **WHEN** 無効な認証情報でログインした場合
- **THEN** エラーメッセージ付きで認証失敗を返す

#### Scenario: 金商法交付書面未読
- **WHEN** sKinsyouhouMidokuFlg が "1"（未読）の場合
- **THEN** 仮想 URL が発行されない旨のエラーを返し、標準 Web での確認を促す

#### Scenario: ログアウト
- **WHEN** セッション終了時にログアウトを実行した場合
- **THEN** 仮想 URL を無効化する

### Requirement: BrokerClient trait による証券 API 抽象化
システムは SHALL `BrokerClient` trait を定義し、`place_order` メソッドの売買方向パラメータとして `Side` enum（Buy/Sell）を使用する。`Side` enum は `as_str()` で `"buy"` / `"sell"` 文字列に変換可能で、`Display` trait を実装する。`place_order` は `second_password` パラメータを受け取り、立花証券 API の `sSecondPassword` フィールドに渡す。

#### Scenario: Side enum による型安全な注文
- **WHEN** `BrokerClient::place_order(Side::Buy, ...)` を呼び出した場合
- **THEN** 立花証券 API の `sBaibaiKubun` に `"3"`（買い）が設定される

#### Scenario: Side::Sell の注文
- **WHEN** `BrokerClient::place_order(Side::Sell, ...)` を呼び出した場合
- **THEN** 立花証券 API の `sBaibaiKubun` に `"1"`（売り）が設定される

#### Scenario: Side enum の文字列変換
- **WHEN** `Side::Buy.as_str()` を呼び出した場合
- **THEN** `"buy"` を返す。`Side::Sell.as_str()` は `"sell"` を返す

### Requirement: REQUEST I/F による注文入力（公式 v4r8 準拠）
システムは SHALL REQUEST I/F（sUrlRequest）を通じて株式現物の指値注文を発注する。リクエスト送信時に JSON キーを圧縮（文字列→数字 1-indexed）し、レスポンス受信時に展開（数字→文字列）する。マッピングテーブルは `mfds_json_api_compress_v4r8.js` の `_pa_col` 配列（941項目）に準拠する。リクエストには p_no（通し番号）と p_sd_date（クライアント時刻）を含める。フィールド名は公式 v4r8 リファレンスに準拠する：`sSizyouC`, `sBaibaiKubun`, `sGenkinShinyouKubun`, `sCondition`, `sOrderPrice`, `sOrderSuryou`, `sOrderExpireDay`, `sGyakusasiOrderType`, `sGyakusasiZyouken`, `sGyakusasiPrice`, `sTatebiType`, `sZyoutoekiKazeiC`, `sSecondPassword`。

#### Scenario: 現物買い注文の発注
- **WHEN** 銘柄コード、指値価格、数量を指定して買い注文を発注した場合
- **THEN** CLMKabuNewOrder で `sBaibaiKubun` = `"3"`, `sGenkinShinyouKubun` = `"0"`, `sZyoutoekiKazeiC` = `"1"`, `sSecondPassword` を含む注文を送信し、注文番号（sOrderNumber）を含む応答を返す

#### Scenario: 現物売り注文の発注
- **WHEN** 保有銘柄に対して指値価格と数量を指定して売り注文を発注した場合
- **THEN** CLMKabuNewOrder で `sBaibaiKubun` = `"1"` の売り注文を送信し、注文番号を含む応答を返す

#### Scenario: 注文詳細レスポンスのパース
- **WHEN** CLMOrderListDetail のレスポンスを受信した場合
- **THEN** 公式フィールド名 `sOrderNumber`, `sIssueCode`, `sOrderStatusCode`, `sBaibaiKubun`, `sOrderPrice`, `sOrderSuryou`, `sYakuzyouPrice`, `sYakuzyouSuryou` でパースする

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
