## Context

DB の中身を確認する手段が散在しており、watchlist_events は表示手段がない。`kabu show` で統一的に DB 内容を閲覧できるようにする。

## Goals / Non-Goals

**Goals:**
- `kabu show <table>` で DB の各テーブルを人間が読みやすい形式で表示
- 既存の重複コマンド（`discover --list`, `history`）を統合・廃止
- human 出力をデフォルトにし、`--format json` で機械可読出力も可能

**Non-Goals:**
- DB の書き込み操作（show は読み取り専用）
- SQL クエリの直接実行

## Decisions

### Decision 1: サブコマンド構造

```
kabu show watchlist              # ウォッチリスト
kabu show events                 # watchlist_events 履歴
kabu show events --ticker 7203   # 特定銘柄のイベント
kabu show positions              # アクティブポジション
kabu show evaluations            # 直近の評価
kabu show evaluations --limit 5  # 件数指定
kabu show stocks                 # 登録済み銘柄
kabu show tables                 # テーブル一覧 + レコード数
```

**Rationale**: `kabu show` のサブコマンドとして各テーブルを指定する。既存の `portfolio positions` は残す（ポートフォリオ操作の文脈で使うため）が、`discover --list` と `history` は `show` に統合して削除する。

### Decision 2: human 出力をデフォルトに

show コマンドは人間が目視確認するためのものなので、`--format` のデフォルトを `human` にする。他のパイプライン系コマンド（scan, eval 等）は従来通り JSON デフォルト。

**実装**: show コマンドのハンドラ内で format を判定し、明示的に指定されていない場合は human を使う。

### Decision 3: `cmd/show.rs` に集約

各サブコマンドのロジックは `cmd/show.rs` に集約。DB クエリは既存の `db/mod.rs` 関数を再利用し、不足分（watchlist_events 取得、テーブル統計）のみ追加。

## Risks / Trade-offs

- [既存コマンド削除による破壊的変更] → `discover --list` と `history` を使っている自動化スクリプトがあれば壊れる。ただし個人ツールなので影響は限定的。
- [show tables のテーブル一覧がハードコード] → schema.rs の ALL_SCHEMAS から動的に取れないため、テーブル名リストをハードコード。テーブル追加時に更新が必要。
