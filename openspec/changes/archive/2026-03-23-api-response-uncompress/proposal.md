## Why

立花証券 e支店 API はレスポンスの JSON キーを数字（1-indexed）に圧縮して返す。例: `"334":"CLMAuthLoginAck"` は `"sCLMID":"CLMAuthLoginAck"` の圧縮形式。現在のパーサーは文字列キー（`sUrlRequest`, `sResultCode` 等）を前提としており、圧縮レスポンスをパースできない。

公式の `mfds_json_api_compress_v4r8.js` にマッピングテーブル（941項目）が提供されており、これを Rust に移植してレスポンスの uncompress を行う。

## What Changes

- `src/tachibana/compress.rs` を新設し、数字キー→文字列キーのマッピングテーブルと uncompress 関数を実装
- リクエスト送信時に compress（文字列キー→数字キー）を適用
- レスポンス受信時に uncompress（数字キー→文字列キー）を適用
- `request.rs` の `parse_response` 前に uncompress 処理を挟む
- マッピングテーブルは `mfds_json_api_compress_v4r8.js` から自動抽出

## Capabilities

### New Capabilities
（なし — 既存機能の修正）

### Modified Capabilities
- `tachibana-api`: リクエスト/レスポンスの圧縮・展開処理を追加

## Impact

- `src/tachibana/compress.rs` — 新規ファイル（マッピングテーブル + compress/uncompress）
- `src/tachibana/mod.rs` — `pub mod compress` 追加、login/send_request_raw で compress/uncompress を呼び出し
- `src/tachibana/request.rs` — `build_request_body` で compress を適用
