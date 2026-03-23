## Context

立花証券 API は `mfds_json_api_compress_v4r8.js` で定義された 941 項目のマッピングテーブルを使い、JSON キーを数字に圧縮する。サーバーはリクエストが圧縮形式であれば圧縮レスポンスを返し、非圧縮であっても圧縮レスポンスを返す場合がある（デモ環境で確認済み）。

マッピング方式:
- 配列 `_pa_col` にキー名がソート済みで格納
- `compress`: キー名 → 配列インデックス + 1（1-indexed）
- `uncompress`: 数字キー - 1 → 配列インデックス → キー名
- 数字でないキーはそのまま通す（混在対応）
- 配列値（`aOrderList` 等）は再帰的に compress/uncompress

## Goals / Non-Goals

**Goals:**
- リクエスト/レスポンスの compress/uncompress を実装し、API 通信を正常化
- 既存のパーサー（`parse_response`, `json_str` 等）はそのまま使えるようにする
- マッピングテーブルを `mfds_json_api_compress_v4r8.js` から正確に移植

**Non-Goals:**
- マッピングテーブルの動的ダウンロード（静的に埋め込む）
- 圧縮モードの on/off 切り替え（常に圧縮を使用）

## Decisions

### Decision 1: マッピングテーブルは静的配列として埋め込む

JS から抽出した 941 項目の配列を Rust の `const` 配列として `compress.rs` に埋め込む。API バージョン更新時には手動で更新が必要だが、頻度は低い（年1-2回）。

### Decision 2: compress/uncompress は serde_json::Value レベルで行う

`serde_json::Value` の `Object` を走査し、キーを変換して新しい `Map` を構築する。配列値は再帰的に処理する。`send_request_raw` と `login` でレスポンスのデコード直後に uncompress を挟む。リクエスト送信時は `build_request_body` の前に compress を適用する。

### Decision 3: 二分探索で compress（名前→ID）を高速化

マッピング配列はアルファベット順にソート済み。compress 時は二分探索で O(log n) で ID を特定する（JS 版と同じアルゴリズム）。uncompress は配列インデックスアクセスなので O(1)。

## Risks / Trade-offs

- [Risk] API バージョン更新でマッピング変更 → JS ファイルから再抽出が必要。docs に手順を記載
- [Trade-off] 毎リクエスト/レスポンスで変換オーバーヘッド → 941 項目の配列アクセスは無視できるレベル
