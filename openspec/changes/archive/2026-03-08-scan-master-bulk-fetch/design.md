## Context

scan コマンドは watchlist の各銘柄に対し、1銘柄ずつ `get_stock_info()` → sleep(1s) → `get_daily_quotes()` を直列実行している。10銘柄で約30秒かかり、うち半分は stock_info API と sleep に費やされている。

J-Quants V2 の `equities/master` エンドポイントは `?code=` パラメータを省略すると全上場銘柄（~4000件）を1回の API 呼び出しで返す。これを stocks テーブルにキャッシュすれば、scan ループから stock_info API を完全に排除できる。

stocks テーブルは evaluations, prices, watchlist, portfolio_positions, fetch_results, trades の6テーブルから FK 参照されている。

## Goals / Non-Goals

**Goals:**
- scan の実行時間を約半分に短縮する
- 全上場銘柄マスターを1回の API 呼び出しでキャッシュする
- `--refresh-master` フラグによる明示的な更新を提供する
- 銘柄間 sleep を 1s → 0.3s に短縮する

**Non-Goals:**
- TTL ベースの自動更新（cron/launchd に委任）
- stocks テーブルのリネーム（FK 変更が大量に必要）
- daily_quotes の一括取得や並列化
- 上場廃止銘柄の自動削除

## Decisions

### D1: マスター取得のトリガー

**選択:** `scan --refresh-master` フラグ

**理由:** アプリ側に TTL やタイマーロジックを持たず、cron/launchd で実行頻度を制御する。週次で `kabu scan --refresh-master --days 60` を回せば十分。

**代替案:**
- TTL ベース自動更新 → アプリに状態管理が増える。cron で十分制御できる
- 専用コマンド `kabu master-sync` → パイプラインに1ステップ追加が必要。scan のオプションの方が自然

### D2: 初回 stocks テーブル空時の振る舞い

**選択:** エラーで案内

**理由:** ユーザーが `--refresh-master` を知る機会になる。暗黙の一括取得は「いつ API が叩かれるか分からない」問題がある。

**メッセージ:** `stocks テーブルが空です。先に kabu scan --refresh-master を実行してください`

### D3: データ競合の解決

**選択:** UPSERT（INSERT OR REPLACE）

**理由:** 既存の `save_stock()` が同じ方式。社名変更・セクター変更を自動反映。ticker にユニーク制約があるため整合性は保たれる。

### D4: sleep 間隔の短縮

**選択:** 銘柄間 sleep を 1s → 0.3s

**理由:** stock_info API 呼び出しが消えるので、ループ内は daily_quotes API のみ。429 リトライ（指数バックオフ）が既にあるため、予防的 sleep は最小限でよい。

## Risks / Trade-offs

- **全銘柄一括取得のレスポンスサイズ** → ~4000件の JSON。数MB 程度で問題なし。ただし無料プランのレート制限に注意
  - Mitigation: `--refresh-master` は明示的実行のみ。頻繁に叩かない設計
- **sleep 短縮による 429 増加** → 0.3s でも連続リクエストになる
  - Mitigation: 既存の指数バックオフリトライ（2s, 4s, 8s）で対応。最悪でも3回リトライで回復
- **上場廃止銘柄が stocks に残る** → UPSERT では削除されない
  - Mitigation: watchlist に入らなければ影響なし。将来的に `--purge-delisted` で対応可能（Non-Goal）
