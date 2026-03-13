# 立花証券 e支店 API 調査サマリー

調査日: 2026-03-12

## API アーキテクチャ

### インタフェース構成

```
顧客側システム                    立花証券側システム
                    HTTPS
  ┌──────────────────────────────────────────────┐
  │                                              │
  │  認証 I/F (非常時接続)                        │
  │    POST https://kabuka.e-shiten.jp/          │
  │         e_api_v4r8/auth/                     │
  │    → sUrlRequest, sUrlEvent,                 │
  │      sUrlEventWebSocket, sUrlMaster,         │
  │      sUrlPrice を取得                         │
  │                                              │
  │  REQUEST I/F (非常時接続・一問一答)            │
  │    仮想URL (REQUEST) — 業務機能               │
  │      注文入力、注文訂正、注文取消              │
  │      注文一覧照会、約定照会                    │
  │      余力照会                                 │
  │    仮想URL (MASTER) — マスタ機能              │
  │    仮想URL (PRICE) — 時価情報機能             │
  │                                              │
  │  EVENT I/F (常時接続・プッシュ)                │
  │    仮想URL (EVENT) — HTTP Chunk 版            │
  │    仮想URL (EVENT-WebSocket) — WebSocket 版   │
  │      注文約定通知                              │
  │      リアルタイム株価                          │
  │      ニュース                                 │
  │      システムステータス                        │
  └──────────────────────────────────────────────┘
```

### 通信プロトコル

- 全て HTTPS (GET/POST)
- リクエスト: URL + JSON クエリパラメータ（URLエンコード済み）
- レスポンス: JSON (Shift-JIS エンコード)
- `p_no`: リクエスト通し番号（毎回インクリメント、シリアル処理を強制）
- `p_sd_date`: クライアント時刻（`YYYY.MM.DD-HH:MM:SS.TTT`、サーバー時刻 ±30秒以内）
- レートリミット: 10リクエスト/秒

### 認証フロー

1. 認証 I/F に POST (userId, password, secondPassword)
2. レスポンスで仮想 URL を取得:
   - `sUrlRequest` — 業務機能用
   - `sUrlMaster` — マスタ機能用
   - `sUrlPrice` — 時価情報用
   - `sUrlEvent` — EVENT I/F (HTTP Chunk)
   - `sUrlEventWebSocket` — EVENT I/F (WebSocket)
3. 仮想 URL は以下で無効化:
   - ログアウト
   - 同一 ID で再ログイン
   - サーバー閉局 (03:30)
4. 電話番号認証が必須（2025-07-04〜）

## 約定検知方法

### EVENT I/F（推奨・公式見解）

**Q12 (公式FAQ):**
> ポーリング利用は想定していません。EVENT I/F(サーバープッシュ)で約定通知を受け、
> それをトリガーに口座状況を確認してください。

EVENT I/F で受信できるイベント種類（`p_evt_cmd` パラメータ）:
- `EC` — 約定通知（推定）
- `ST` — システムステータス
- `KP` — 株価
- `FD` — 歩み値等
- `NS` — ニュース
- `SS` — セッションステータス
- `US` — ユーザーステータス

### 注文→約定のシーケンス（公式 PDF p.16-17）

```
execute (REQUEST I/F)              EVENT I/F (WebSocket)
    │                                    │
    ├─ 余力確認 ───────→                  │
    │    ←─── 応答 ────┤                  │
    │                                    │
    ├─ 注文入力 ───────→                  │
    │    ←─── 応答 ────┤                  │
    │                    ├──→ 通知: 注文状態変更
    │                    ├──→ 通知: 注文受付 (or 受付エラー)
    │                    │
    │               (市場で約定)
    │                    │
    │                    ├──→ 通知: 約定成立 ← ★これが約定検知
    │                    │
```

### 注意事項

- REQUEST I/F と EVENT I/F は別接続。タイミングが前後する可能性あり
- EVENT I/F は 1 顧客 1 接続（WebSocket / HTTP Chunk どちらか一方）
- 訂正受付/訂正完了の通知順序が逆になることがある
- 同一 ID で再ログインすると既存セッションが無効化される

## REQUEST I/F 主要コマンド

### 注文系

| コマンド (sCLMID) | 機能 |
|-------------------|------|
| CLMKabuNewOrder | 株式新規注文 |
| CLMKabuCorrectOrder | 株式注文訂正（推定）|
| CLMKabuCancelOrder | 株式注文取消（推定）|

### 照会系

| コマンド (sCLMID) | 機能 |
|-------------------|------|
| CLMOrderList | 注文一覧照会 |
| CLMOrderListDetail | 注文約定詳細（注文番号+営業日で照会）|
| CLMShinyouTategyokuList | 信用建玉一覧 |

### レスポンス共通項目

- `p_errno`: エラー番号（0=正常、-2=パラメータエラー、2=仮想URL無効、8=時刻ずれ）
- `p_err`: エラーテキスト
- `sResultCode`: 結果コード（0=OK）
- `sResultText`: 結果テキスト

## CLMOrderListDetail レスポンス（主要フィールド）

| フィールド | 説明 |
|-----------|------|
| sIssueCode | 銘柄コード |
| sOrderSizyouC | 市場（00=東証）|
| sOrderBaibaiKubun | 売買区分（1=売、3=買）|
| sGenkinSinyouKubun | 現金信用区分（0=現物）|
| sOrderOrderPriceKubun | 注文値段区分（1=成行、2=指値）|
| sOrderOrderPrice | 注文単価 |
| sOrderOrderSuryou | 注文株数 |
| sOrderCurrentSuryou | 有効株数 |
| sOrderStatusCode | 状態コード（→ order-status-codes.md 参照）|
| sOrderOrderExpireDay | 有効期限 (YYYYMMDD) |
| sYakuzyouPrice | 約定単価 |
| sYakuzyouSuryou | 約定株数 |
| sBaiBaiDaikin | 売買代金 |
| sBaiBaiTesuryo | 手数料 |
| sShouhizei | 消費税 |
| aYakuzyouSikkouList | 約定失効リスト（配列）|

## kekekabu 設計への影響

### 約定検知方式の結論

**Option C（純粋ポーリング）は立花証券が非推奨。**

日次バッチ + 指値注文の運用では以下の2段階アプローチを検討:

1. **Phase 1: execute 内 短命 WebSocket（Option A 変形）**
   - execute 時に WebSocket 接続
   - 注文発注後、一定時間（例: 60秒）約定通知を待つ
   - タイムアウトしたら pending のまま切断
   - 翌日の execute 冒頭で REQUEST I/F の約定照会で settle

2. **Phase 2: daemon 化（Option B、必要になったら）**
   - WebSocket 常時接続で約定通知をリアルタイム処理
   - launchd KeepAlive で管理

### REQUEST I/F 約定照会（フォールバック用）

EVENT I/F が切断された場合や、前日の pending 注文確認用:
- `CLMOrderListDetail` で注文番号 + 営業日を指定して照会
- `sOrderStatusCode` で約定状態を判定
