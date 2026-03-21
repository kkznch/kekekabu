## MODIFIED Requirements

### Requirement: REQUEST I/F による注文入力
システムは SHALL リクエスト送信時に JSON キーを圧縮（文字列→数字 1-indexed）し、レスポンス受信時に展開（数字→文字列）する。マッピングテーブルは `mfds_json_api_compress_v4r8.js` の `_pa_col` 配列（941項目）に準拠する。

#### Scenario: リクエストの圧縮
- **WHEN** REQUEST I/F にリクエストを送信する場合
- **THEN** JSON キーを `_pa_col` 配列のインデックス + 1（1-indexed）に変換して送信する。配列値は再帰的に圧縮する

#### Scenario: レスポンスの展開
- **WHEN** API からレスポンスを受信した場合
- **THEN** 数字キーを `_pa_col` 配列で逆引きして文字列キーに復元する。数字でないキーはそのまま通す。配列値は再帰的に展開する

#### Scenario: 認証レスポンスの展開
- **WHEN** 認証 I/F からログインレスポンスを受信した場合
- **THEN** 数字キーを展開し、`sUrlRequest`, `sUrlEventWebSocket` 等の文字列キーでアクセス可能にする

### Requirement: 立花証券 API 認証
システムは SHALL 認証 I/F に HTTPS POST で圧縮された JSON body を送信してログインし、レスポンスを展開して仮想 URL を取得する。

#### Scenario: ログイン成功（圧縮レスポンス）
- **WHEN** 有効な認証情報で圧縮リクエストを送信した場合
- **THEN** 圧縮されたレスポンスを展開し、sUrlRequest, sUrlEventWebSocket 等を含むセッション情報を返す
