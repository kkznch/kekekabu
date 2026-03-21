## MODIFIED Requirements

### Requirement: 立花証券 API 認証
システムは SHALL 立花証券 e支店 API の認証 I/F に HTTPS POST で `sCLMID: "CLMAuthLoginRequest"` を含む Shift-JIS エンコードされた JSON body を送信してログインし、仮想 URL（sUrlRequest, sUrlEvent, sUrlEventWebSocket）を取得してセッションを管理する。

#### Scenario: ログイン成功
- **WHEN** 有効な userId, password, secondPassword で認証 I/F に POST リクエストした場合
- **THEN** sUrlRequest, sUrlEvent, sUrlEventWebSocket を含むセッション情報を返す

#### Scenario: 認証失敗
- **WHEN** 無効な認証情報でログインした場合
- **THEN** エラーメッセージ付きで認証失敗を返す

#### Scenario: 金商法交付書面未読
- **WHEN** sKinsyouhouMidokuFlg が "1"（未読）の場合
- **THEN** 仮想 URL が発行されない旨のエラーを返し、標準 Web での確認を促す

#### Scenario: ログアウト
- **WHEN** セッション終了時にログアウトを実行した場合
- **THEN** `sCLMID: "CLMAuthLogoutRequest"` を POST で送信し、仮想 URL を無効化する

### Requirement: REQUEST I/F による注文入力
システムは SHALL REQUEST I/F（sUrlRequest）に対して HTTPS POST で Shift-JIS エンコードされた JSON body を送信して株式現物の注文を発注する。リクエストには p_no（通し番号）と p_sd_date（クライアント時刻）を含める。Content-Type ヘッダには `application/json; charset=Shift_JIS` を指定する。

#### Scenario: POST による注文送信
- **WHEN** CLMKabuNewOrder の注文リクエストを送信する場合
- **THEN** JSON を Shift-JIS エンコードして POST body として送信し、注文番号を含む応答を返す

#### Scenario: p_no のインクリメント
- **WHEN** 連続してリクエストを送信する場合
- **THEN** p_no を毎回 +1 以上インクリメントして送信する

#### Scenario: Shift-JIS レスポンスのデコード
- **WHEN** API からレスポンスを受信した場合
- **THEN** Shift-JIS エンコードのレスポンスを UTF-8 に変換してパースする
