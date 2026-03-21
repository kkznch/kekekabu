## Context

立花証券 e支店 API v4r8 (2025-09-27) で認証 I/F と REQUEST I/F に HTTPS POST サポートが追加された。既存実装は GET + URL クエリパラメータ方式。EVENT I/F は引き続き GET のみ（WebSocket ハンドシェイク）。

## Goals / Non-Goals

**Goals:**
- 認証 I/F と REQUEST I/F を POST 方式に移行
- sCLMID 名を公式リファレンスに合わせる

**Non-Goals:**
- EVENT I/F の変更（WebSocket は変更なし）
- GET 方式の後方互換サポート

## Decisions

### Decision 1: POST body を Shift-JIS エンコードする

API は Shift-JIS エンコーディング。GET 時は URL エンコードで ASCII 化されていたが、POST body では Content-Type に charset=Shift_JIS を指定し、body 自体を Shift-JIS でエンコードして送信する。`encoding_rs::SHIFT_JIS.encode()` を使用。

### Decision 2: build_request_url を完全に置換

GET 用の `build_request_url()` を削除し、POST 用の `build_request_body()` に置換。両方を残す意味がない（v4r8 以降は POST のみ使用）。

### Decision 3: CLMAuthLoginRequest を明示的に送信

GET 時代は認証 URL にリクエストすれば暗黙的にログインと判定されていたが、POST では `sCLMID: "CLMAuthLoginRequest"` を明示的に JSON body に含める。

## Risks / Trade-offs

- [Risk] v4r8 未満の環境では動作しない → v4r7 は既にリタイア済みのため問題なし
