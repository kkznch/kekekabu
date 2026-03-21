## 1. REQUEST I/F の POST 移行

- [x] 1.1 `request.rs` の `build_request_url()` を `build_request_body()` に置換（Shift-JIS エンコード）
- [x] 1.2 `mod.rs` の `login()` を POST + JSON body に変更
- [x] 1.3 `mod.rs` の `send_request_raw()` を POST に変更
- [x] 1.4 `mod.rs` の `logout()` を POST に変更
- [x] 1.5 Content-Type ヘッダに `application/json; charset=Shift_JIS` を設定

## 2. sCLMID 名の修正

- [x] 2.1 ログインリクエストに `sCLMID: "CLMAuthLoginRequest"` を追加
- [x] 2.2 ログアウトの `CLMLogout` を `CLMAuthLogoutRequest` に変更

## 3. テスト

- [x] 3.1 `test_build_request_url` を `test_build_request_body` に更新
- [x] 3.2 `just ci` で全テスト通過確認
