## Context

execute コマンドは eval 結果に基づいて売買シグナルを生成するが、実際の注文発注はスタブ。立花証券 e支店 API（v4r8）の調査が完了し、以下が判明している:

- REQUEST I/F: HTTPS GET/POST の一問一答方式（注文入力、約定照会、余力照会）
- EVENT I/F: WebSocket 常時接続のプッシュ方式（約定通知、株価配信）
- 認証: 認証 I/F → 仮想 URL 取得 → 各機能にアクセス
- 制約: ポーリング非推奨（公式 FAQ Q12）、p_no インクリメント必須、Shift-JIS レスポンス

既存の `portfolio::buy/sell` は約定済みを前提に DB 記録する関数として実装済み。

## Goals / Non-Goals

**Goals:**
- 立花証券 API 経由で指値注文を発注し、約定を検知して portfolio に記録する
- orders テーブルで注文ライフサイクルを管理し、べき等性を担保する
- dry-run モードでは API 未接続でも従来どおりシグナル出力のみ
- execute 冒頭で前回 pending 注文を settle（約定確認）する

**Non-Goals:**
- daemon 化（常時 WebSocket 接続）— 将来の拡張として残す
- 信用取引対応 — 現物取引のみ
- 逆指値注文 — 通常指値のみ
- 成行注文 — 指値のみ（将来追加可能な設計にはする）
- リアルタイム株価配信の利用
- 口座残高からの budget_context 動的生成（WANT.md の別タスク）

## Decisions

### D1: API クライアントモジュール構造

`src/tachibana/` を新設し、関心ごとに分割:

```
src/tachibana/
  mod.rs          — TachibanaClient 構造体、認証、セッション管理
  request.rs      — REQUEST I/F ヘルパー（JSON 構築、URL エンコード、p_no 管理）
  order.rs        — 注文入力、注文照会、約定照会のコマンド
  event.rs        — EVENT I/F WebSocket 接続、約定通知パース
```

**理由:** 立花 API は独特な仕様（GET + JSON クエリ、Shift-JIS、p_no シリアル制御）なので、それを隠蔽するクライアント層が必要。LLM バックエンドと同様にトレイト抽象化はせず、単一実装として直接的に書く。

**代替案:** 単一ファイル `src/tachibana.rs` — API が十分複雑なため分割が妥当と判断。

### D2: execute のフロー

```
kabu execute [--dry-run]
  │
  ├─ 1. settle: pending 注文の約定確認
  │     ├─ orders テーブルから status=pending を取得
  │     ├─ 立花 API にログイン（settle 対象がある場合のみ）
  │     ├─ CLMOrderListDetail で各注文の状態を照会
  │     ├─ 約定 → portfolio::buy/sell + orders を filled に更新
  │     ├─ 失効 → orders を expired に更新
  │     └─ まだ pending → そのまま（翌日も settle 対象）
  │
  ├─ 2. シグナル生成（既存ロジック）
  │     ├─ 当日の evaluations を取得
  │     ├─ Buy/Sell シグナルを判定
  │     └─ べき等チェック: 同じ eval に対する注文が既にあればスキップ
  │
  ├─ 3. 発注（dry-run でなければ）
  │     ├─ 立花 API にログイン（未ログインなら）
  │     ├─ 各シグナルに対して CLMKabuNewOrder で指値注文
  │     ├─ orders テーブルに pending で記録
  │     └─ request_id で重複防止
  │
  ├─ 4. 短時間 WebSocket 約定待ち（オプション）
  │     ├─ EVENT I/F WebSocket に接続
  │     ├─ 最大 N 秒間約定通知を待つ
  │     ├─ 約定通知受信 → settle と同様に処理
  │     └─ タイムアウト → pending のまま（翌日 settle）
  │
  └─ 5. ログアウト + 結果出力
```

**理由:** settle を execute 冒頭に置くことで、別コマンド不要。日次バッチの自然なフローに合致。

**代替案:** settle を独立コマンドにする — コマンド増加を避けるため execute に統合。

### D3: orders テーブル設計

```sql
CREATE TABLE IF NOT EXISTS orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    stock_id INTEGER NOT NULL REFERENCES stocks(id),
    side TEXT NOT NULL CHECK(side IN ('buy', 'sell')),
    order_type TEXT NOT NULL DEFAULT 'limit',
    price TEXT NOT NULL,
    quantity TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK(status IN ('pending', 'filled', 'partial', 'cancelled', 'expired', 'rejected')),
    tachibana_order_id TEXT,
    request_id TEXT NOT NULL UNIQUE,
    filled_price TEXT,
    filled_quantity TEXT,
    filled_at TEXT,
    evaluation_id INTEGER REFERENCES evaluations(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**理由:** 既存テーブルとの一貫性（money as TEXT, datetime as TEXT）。request_id UNIQUE でべき等性を保証。evaluation_id で eval → order の紐付け。

### D4: べき等性の request_id

フォーマット: `{date}-{ticker}-{side}-{evaluation_id}`
例: `2026-03-13-7203-buy-42`

同じ eval から同日に同じ銘柄・同じ方向の注文は1回だけ。INSERT OR IGNORE で重複排除。

**理由:** 立花 API 側にべき等性保証がないため、クライアント側で防ぐ必要がある。

### D5: config 拡張

```toml
[tachibana]
user_id = "..."
password = "..."
second_password = "..."
# url = "https://kabuka.e-shiten.jp/e_api_v4r8"  # デフォルト
```

環境変数: `TACHIBANA_USER_ID`, `TACHIBANA_PASSWORD`, `TACHIBANA_SECOND_PASSWORD`

**理由:** 既存の api セクション（jquants, anthropic）と同じパターン。

### D6: WebSocket 約定待ちのタイムアウト

デフォルト 30 秒。config でオーバーライド可能: `tachibana.event_timeout_secs = 60`

指値注文は即座に約定しないことが多いため、短めのタイムアウトで切断して翌日 settle に任せる。

**理由:** 日次バッチの実行時間を圧迫しない。

## Risks / Trade-offs

- **[仮想 URL 有効期限]** settle と発注が同一セッションで行われる前提だが、サーバー閉局（03:30）をまたぐ場合は再ログインが必要 → ログイン失敗時のリトライとエラーハンドリングを実装
- **[部分約定]** 指値 100 株中 50 株だけ約定する可能性 → Phase 1 では partial を orders.status として記録するが、portfolio への反映は全量約定時のみ。partial の settle は翌日以降に再確認
- **[通知順序の逆転]** 立花 API は訂正受付/完了の通知順序が逆になることがある → 注文状態の最終判定は REQUEST I/F の照会結果を正とする
- **[電話認証]** 2025-07-04 以降、電話番号認証が必須 → 初回セットアップ手順をドキュメント化する（自動化は困難）
- **[Shift-JIS]** レスポンスが Shift-JIS エンコード → `encoding_rs` crate で変換
