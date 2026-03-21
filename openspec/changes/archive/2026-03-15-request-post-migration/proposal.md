## Why

立花証券 e支店 API v4r8 で HTTPS POST がサポートされた。GET 方式は URL クエリパラメータに JSON を載せるため URL 長制限やログ漏洩のリスクがある。POST に移行し、sCLMID 名も公式リファレンスに準拠させる。

## What Changes

- 認証 I/F および REQUEST I/F の HTTP メソッドを GET → POST に変更
- リクエストボディを Shift-JIS エンコードした JSON として送信
- `build_request_url()` を `build_request_body()` に置換
- `CLMLogout` → `CLMAuthLogoutRequest` に修正（公式sCLMID名に準拠）
- ログインリクエストに `sCLMID: "CLMAuthLoginRequest"` を明示追加

## Capabilities

### New Capabilities

（なし）

### Modified Capabilities
- `tachibana-api`: REQUEST I/F の通信方式を GET→POST に変更、sCLMID 名を公式に修正

## Impact

- `src/tachibana/mod.rs` — login(), send_request_raw(), logout() の HTTP メソッドと body 構築を変更
- `src/tachibana/request.rs` — build_request_url() を build_request_body() に置換、テスト更新
