## Context

discover コマンドは現在、LLM に「有望銘柄を教えて」とゼロベースで聞き、返ってきた `candidates` リストとコード側で機械的に差分を取っている。LLM は現在の watchlist を知らないため、意図的な入れ替え判断ができない。また変更履歴が残らないため、銘柄の出入りの経緯を追跡できない。

## Goals / Non-Goals

**Goals:**
- discover プロンプトに現在のウォッチリストを含め、LLM が keep/add/remove を明示的に判断できるようにする
- watchlist の変更イベントを記録し、履歴を追跡可能にする

**Non-Goals:**
- watchlist_events の UI/CLI 表示機能（後日追加）
- イベントソーシング（現状の watchlist テーブルは残し、events は追加ログとして運用）

## Decisions

### Decision 1: LLM レスポンスを keep/add/remove のアクション別構造にする

現在の `{ "candidates": [...] }` から以下に変更:

```json
{
  "keep": [
    { "ticker": "7203", "reason": "ROE改善トレンド継続中" }
  ],
  "add": [
    { "ticker": "6758", "name": "ソニー", "reason": "新規カタリスト発生" }
  ],
  "remove": [
    { "ticker": "9984", "reason": "PBR基準を満たさなくなった" }
  ]
}
```

**Rationale**: LLM の判断意図が明確になり、「たまたま言及しなかった」と「意図的に外した」を区別できる。

**Alternative**: 現在の candidates 方式を維持しつつ LLM に「前回のリストはこれ」と渡す → LLM が返さなかった銘柄の扱いが曖昧なまま。

### Decision 2: watchlist_events テーブルを追加（イベントログ方式）

現在の watchlist テーブル（スナップショット）はそのまま残し、`watchlist_events` をイベントログとして追加する。

```sql
CREATE TABLE IF NOT EXISTS watchlist_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ticker TEXT NOT NULL,
    action TEXT NOT NULL CHECK(action IN ('add', 'remove', 'keep')),
    reason TEXT,
    discovered_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

**Rationale**: フルイベントソーシングは過剰。watchlist テーブルの現在状態は既存コード（eval, fetch 等）が参照しているのでそのまま維持し、events は純粋なログとして追加するのが最もシンプル。

### Decision 3: プロンプトへのウォッチリスト注入方式

`build_discover_prompt` に `watchlist_context: Option<&str>` パラメータを追加。現在のウォッチリストを ticker + name のリストとしてプロンプトに埋め込む。

watchlist が空の場合は「現在の追跡銘柄なし」としてプロンプトを構築し、add のみを期待する。

## Risks / Trade-offs

- [LLM が keep/add/remove の構造を守らない] → `extract_json` で JSON 抽出 + serde のデフォルト値で対応。keep/add/remove の各フィールドを `#[serde(default)]` にして、欠落しても空配列として扱う。
- [keep に含まれず remove にも含まれない銘柄] → LLM が言及し忘れた銘柄は変更なし（watchlist に残す）として扱う。安全側に倒す。
- [watchlist_events の肥大化] → 当面は問題にならない（1日1回 × 10-20銘柄程度）。将来必要なら retention policy を追加。
