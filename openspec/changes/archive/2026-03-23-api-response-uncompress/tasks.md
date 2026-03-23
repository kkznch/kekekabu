## 1. マッピングテーブルの抽出

- [x] 1.1 `mfds_json_api_compress_v4r8.js` から 941 項目の `_pa_col` 配列を抽出し Rust の const 配列に変換
- [x] 1.2 `src/tachibana/compress.rs` を新設し、マッピングテーブルと compress/uncompress 関数を実装

## 2. compress/uncompress 実装

- [x] 2.1 `uncompress(value: &Value) -> Value` — 数字キー（1-indexed）→文字列キーに再帰的に変換
- [x] 2.2 `compress(value: &Value) -> Value` — 文字列キー→数字キー（1-indexed）に二分探索で変換
- [x] 2.3 数字でないキー、配列外の数字キーはそのまま通す（混在対応）

## 3. API クライアントへの組み込み

- [x] 3.1 `mod.rs` に `pub mod compress` を追加
- [x] 3.2 `login()` — リクエスト送信前に compress、レスポンス受信後に uncompress を適用
- [x] 3.3 `send_request_raw()` — リクエスト送信前に compress、レスポンス受信後に uncompress を適用
- [x] 3.4 debug ログ出力を uncompress 後の body に変更

## 4. テスト

- [x] 4.1 compress/uncompress の単体テスト（sCLMID ↔ 334, sUrlRequest ↔ 872 等）
- [x] 4.2 配列値の再帰的 compress/uncompress テスト
- [x] 4.3 混在キー（数字 + 文字列）のテスト
- [x] 4.4 `just ci` で全テスト通過確認
